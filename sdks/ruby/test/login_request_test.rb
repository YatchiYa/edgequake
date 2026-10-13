# frozen_string_literal: true

require "minitest/autorun"
require "socket"
require_relative "../lib/edgequake"

class LoginRequestTest < Minitest::Test

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
    yield EdgeQuake::AuthService.new(EdgeQuake::HttpHelper.new(config))
    worker.value
  ensure
    worker&.kill if worker&.alive?
    server&.close
  end

  def test_login_sends_username_wire_field
    body = capture_body { |service| service.login(email: "alice@example.test", password: "test-password") }
    assert_equal({"username" => "alice@example.test", "password" => "test-password"}, body)
  end
end
