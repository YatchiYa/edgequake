# frozen_string_literal: true

require "minitest/autorun"
require "socket"
require_relative "../lib/edgequake"

class PathEncodingTest < Minitest::Test
  def capture_path
    server = TCPServer.new("127.0.0.1", 0)
    worker = Thread.new do
      socket = server.accept
      line = socket.gets
      while (header = socket.gets) && header != "\r\n"
      end
      socket.write("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
      socket.close
      line.split[1]
    end
    config = EdgeQuake::Config.new(base_url: "http://127.0.0.1:#{server.addr[1]}", timeout: 2)
    yield EdgeQuake::HttpHelper.new(config)
    worker.value
  ensure
    server&.close
    worker&.kill if worker&.alive?
  end

  def test_entity_names_round_trip_as_path_segments
    ["HELLO WORLD", "A+B", "café 東京", "A/B"].each do |name|
      path = capture_path { |http| EdgeQuake::EntityService.new(http).get(name: name) }
      segment = path.delete_prefix("/api/v1/graph/entities/")
      assert_equal name, URI::DEFAULT_PARSER.unescape(segment).force_encoding(Encoding::UTF_8)
      refute_includes segment, "+"
      refute_includes segment, "/"
    end
  end

  def test_query_spaces_keep_form_encoding
    path = capture_path { |http| EdgeQuake::EntityService.new(http).exists?(name: "HELLO WORLD+A") }
    assert_equal [["entity_name", "HELLO WORLD+A"]], URI.decode_www_form(URI.parse(path).query)
  end
end
