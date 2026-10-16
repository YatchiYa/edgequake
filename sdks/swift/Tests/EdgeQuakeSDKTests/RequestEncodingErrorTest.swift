import Foundation
import XCTest
@testable import EdgeQuakeSDK

private enum RequestEncodingFailure: Error { case deliberate }
private struct FailingRequestBody: Encodable {
    func encode(to encoder: Encoder) throws { throw RequestEncodingFailure.deliberate }
}

final class RequestEncodingErrorTest: XCTestCase {
    func testEncodingFailurePreventsNetworkRequests() async throws {
        for method in ["POST", "PUT", "PATCH"] {
            let http = mockHelper()
            do {
                _ = try await http.call(method, path: "/api/v1/documents", body: FailingRequestBody())
                XCTFail("\(method) discarded the encoding failure")
            } catch RequestEncodingFailure.deliberate {
                XCTAssertTrue(MockURLProtocol.requestHistory.isEmpty)
            }
        }
    }

    func testValidAndDefaultBodiesRemainJSON() async throws {
        let http = mockHelper()
        _ = try await http.call("POST", path: "/api/v1/documents", body: TextUploadRequest(title: "Report", content: "Text"))
        let payload = try XCTUnwrap(MockURLProtocol.lastRequest?.body)
        let decoded = try XCTUnwrap(JSONSerialization.jsonObject(with: payload) as? [String: String])
        XCTAssertEqual(decoded["file_type"], "txt")
        XCTAssertEqual(decoded["content"], "Text")
        _ = try await http.call("PATCH", path: "/api/v1/documents")
        XCTAssertEqual(MockURLProtocol.lastRequest?.body, Data("{}".utf8))
    }
}
