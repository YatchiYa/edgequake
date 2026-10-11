# frozen_string_literal: true

require "json"
require "minitest/autorun"

class OperationsManifestTest < Minitest::Test
  def test_every_http_operation_path_and_generated_verb
    root = File.expand_path("../../..", __dir__)
    ops = JSON.parse(File.read(File.join(root, "sdks/operation-manifest.json")))
    generated = File.read(File.join(root, "sdks/ruby/lib/edgequake/operations_generated.rb"))
    blob = Dir[File.join(root, "sdks/ruby/lib/**/*.rb")].map { |path| File.read(path) }.join
    ops.each do |op|
      if op["kind"] == "websocket"
        assert_includes generated, op["path"]
        next
      end
      assert_includes blob, op["path"]
      assert_includes generated, "#{op["method"]} #{op["path"]}" if op["missing"]["ruby"]
    end
  end
end
