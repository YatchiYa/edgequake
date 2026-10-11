# frozen_string_literal: true

require "minitest/autorun"
require "socket"
require_relative "../lib/edgequake"

class BulkRequestTest < Minitest::Test
  IDS = ["550e8400-e29b-41d4-a716-446655440000"].freeze

  def capture_body
    server = TCPServer.new("127.0.0.1", 0)
    worker = Thread.new do
      socket = server.accept
      socket.gets
      headers = {}
      while (line = socket.gets) && line != "\r\n"
        key, value = line.split(":", 2)
        headers[key.downcase] = value.strip
      end
      body = JSON.parse(socket.read(headers.fetch("content-length").to_i))
      socket.write("HTTP/1.1 200 OK\r\nContent-Length: 14\r\nConnection: close\r\n\r\n{\"affected\":1}")
      socket.close
      body
    end
    config = EdgeQuake::Config.new(base_url: "http://127.0.0.1:#{server.addr[1]}", timeout: 2)
    yield EdgeQuake::ConversationService.new(EdgeQuake::HttpHelper.new(config))
    worker.value
  ensure
    worker&.kill if worker&.alive?
    server&.close
  end

  def test_bulk_requests_use_server_conversation_ids_field
    deleted = capture_body { |service| service.bulk_delete(ids: IDS) }
    archived = capture_body { |service| service.bulk_archive(ids: IDS) }
    moved = capture_body { |service| service.bulk_move(ids: IDS, folder_id: nil) }
    assert_equal({"conversation_ids" => IDS}, deleted)
    assert_equal({"conversation_ids" => IDS, "archive" => true}, archived)
    assert_equal({"conversation_ids" => IDS, "folder_id" => nil}, moved)
  end

  def test_bulk_archive_can_clear_archive_state
    body = capture_body { |service| service.bulk_archive(ids: IDS, archive: false) }
    assert_equal({"conversation_ids" => IDS, "archive" => false}, body)
  end
end
