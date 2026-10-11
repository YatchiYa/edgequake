import XCTest

final class OperationsManifestTest: XCTestCase {
    func testEveryHttpOperationPathAndGeneratedVerb() throws {
        let root = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        let ops = try JSONSerialization.jsonObject(with: Data(contentsOf: root.appendingPathComponent("sdks/operation-manifest.json"))) as! [[String: Any]]
        let generated = try String(contentsOf: root.appendingPathComponent("sdks/swift/Sources/EdgeQuakeSDK/OperationsGenerated.swift"))
        var blob = ""
        let src = root.appendingPathComponent("sdks/swift/Sources")
        if let enumerator = FileManager.default.enumerator(at: src, includingPropertiesForKeys: nil) {
            for case let url as URL in enumerator where url.pathExtension == "swift" {
                blob += (try? String(contentsOf: url)) ?? ""
            }
        }
        for op in ops {
            if op["kind"] as? String == "websocket" {
                XCTAssertTrue(generated.contains(op["path"] as! String))
                continue
            }
            let path = op["path"] as! String
            XCTAssertTrue(blob.contains(path), path)
            let missing = op["missing"] as! [String: Bool]
            if missing["swift"] == true {
                let method = op["method"] as! String
                XCTAssertTrue(generated.contains("\(method) \(path)"), path)
            }
        }
    }
}
