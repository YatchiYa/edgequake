# frozen_string_literal: true

require "json"
require "net/http"
require "uri"

module EdgeQuake
  class HttpHelper
    def initialize(config)
      @config = config
    end

    def get(path) = parse(request_raw(:get, path))
    def post(path, body = nil) = parse(request_raw(:post, path, body))
    def put(path, body = nil) = parse(request_raw(:put, path, body))
    def patch(path, body = nil) = parse(request_raw(:patch, path, body))
    def delete(path, body = nil) = parse(request_raw(:delete, path, body))
    def get_raw(path) = request_raw(:get, path)
    def call_raw(method, path, body = nil) = request_raw(method.to_sym, path, body)
    def base_url = @config.base_url

    private

    def parse(raw)
      return {} if raw.nil? || raw.empty?

      JSON.parse(raw)
    end

    def request_raw(method, path, body = nil)
      uri = URI.parse("#{@config.base_url}#{path}")
      request = {
        get: Net::HTTP::Get,
        post: Net::HTTP::Post,
        put: Net::HTTP::Put,
        patch: Net::HTTP::Patch,
        delete: Net::HTTP::Delete,
        head: Net::HTTP::Head,
        options: Net::HTTP::Options
      }.fetch(method).new(uri)
      headers.each { |key, value| request[key] = value }
      request.body = JSON.generate(body) unless body.nil?

      response = Net::HTTP.start(
        uri.host, uri.port,
        use_ssl: uri.scheme == "https",
        open_timeout: @config.timeout,
        read_timeout: @config.timeout
      ) { |http| http.request(request) }
      raw = response.body.to_s
      return raw if response.is_a?(Net::HTTPSuccess)

      raise ApiError.new(
        "HTTP #{response.code}: #{raw}",
        status_code: response.code.to_i,
        response_body: raw
      )
    end

    def headers
      result = { "Accept" => "application/json", "Content-Type" => "application/json" }
      result["X-API-Key"] = @config.api_key if @config.api_key
      result["X-Tenant-ID"] = @config.tenant_id if @config.tenant_id
      result["X-User-ID"] = @config.user_id if @config.user_id
      result["X-Workspace-ID"] = @config.workspace_id if @config.workspace_id
      result
    end
  end
end
