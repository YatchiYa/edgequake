# frozen_string_literal: true

require "minitest/autorun"
require "socket"
require_relative "../lib/edgequake"

class MergeRequestTest < Minitest::Test

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
    yield EdgeQuake::EntityService.new(EdgeQuake::HttpHelper.new(config))
    worker.value
  ensure
    worker&.kill if worker&.alive?
    server&.close
  end

  def test_merge_uses_server_entity_fields
    body = capture_body { |service| service.merge(source_name: "ALICE", target_name: "BOB") }
    assert_equal({"source_entity" => "ALICE", "target_entity" => "BOB"}, body)
  end
end
