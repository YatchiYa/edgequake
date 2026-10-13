# frozen_string_literal: true

require "minitest/autorun"
require "socket"
require_relative "../lib/edgequake"

class StreamConsumptionTest < Minitest::Test
  def with_response(response, status: 200)
    server = TCPServer.new("127.0.0.1", 0)
    worker = Thread.new do
      socket = server.accept
      path = socket.gets.split[1]
      headers = {}
      while (line = socket.gets) && line != "\r\n"
        key, value = line.split(":", 2)
        headers[key.downcase] = value.strip
      end
      body = JSON.parse(socket.read(headers.fetch("content-length").to_i))
      socket.write("HTTP/1.1 #{status} Test\r\nContent-Type: text/event-stream\r\nContent-Length: #{response.bytesize}\r\nConnection: close\r\n\r\n#{response}")
      socket.close
      [path, body]
    end
    http = EdgeQuake::HttpHelper.new(EdgeQuake::Config.new(base_url: "http://127.0.0.1:#{server.addr[1]}", timeout: 2))
    yield http, worker
  ensure
    worker&.kill if worker&.alive?
    server&.close
  end

  def test_query_and_chat_enumerate_sse_json_payloads
    events = "\uFEFF: keepalive\r\nevent: token\r\ndata: {\"type\":\"token\",\r\ndata: \"content\":\"héllo\"}\r\n\r\ndata: {\"type\":\"done\"}\r\n\r\ndata: [DONE]\r\n\r\n"
    [EdgeQuake::QueryService, EdgeQuake::ChatService].each do |klass|
      with_response(events) do |http, worker|
        service = klass.new(http)
        stream = klass == EdgeQuake::QueryService ? service.stream(query: "hi") : service.stream(message: "hi")
        assert_instance_of Enumerator, stream
        assert_equal [{"type" => "token", "content" => "héllo"}, {"type" => "done"}], stream.to_a
        path, body = worker.value
        expected = klass == EdgeQuake::QueryService ? "/api/v1/query/stream" : "/api/v1/chat/completions/stream"
        assert_equal expected, path
        assert_equal "hi", body[klass == EdgeQuake::QueryService ? "query" : "message"]
      end
    end
  end

  def test_invalid_json_payload_is_reported
    with_response("data: invalid\n\n") do |http, worker|
      assert_raises(JSON::ParserError) { EdgeQuake::QueryService.new(http).stream(query: "hi").to_a }
      worker.value
    end
  end

  def test_stream_keeps_http_error_status
    with_response('{"error":"invalid"}', status: 422) do |http, worker|
      error = assert_raises(EdgeQuake::ApiError) { EdgeQuake::QueryService.new(http).stream(query: "hi").to_a }
      assert_equal 422, error.status_code
      worker.value
    end
  end
end
