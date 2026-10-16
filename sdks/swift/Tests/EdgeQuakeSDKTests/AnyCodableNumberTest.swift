import Foundation
import XCTest
@testable import EdgeQuakeSDK

final class AnyCodableNumberTest: XCTestCase {
    func testNativeNumericTypesEncodeAsNumbers() throws {
        let values: [Any] = [Int8(-8), Int16(-16), Int32(-32), Int64(-64), UInt(7), UInt8(8), UInt16(16), UInt32(32), UInt64.max, Float(1.25)]
        let expected = ["-8", "-16", "-32", "-64", "7", "8", "16", "32", "18446744073709551615", "1.25"]
        for (value, text) in zip(values, expected) {
            let data = try JSONEncoder().encode(AnyCodable(value))
            XCTAssertEqual(String(decoding: data, as: UTF8.self), text)
        }
    }

    func testLargeUnsignedIntegerDecodesWithoutRounding() throws {
        let data = Data("18446744073709551615".utf8)
        let decoded = try JSONDecoder().decode(AnyCodable.self, from: data)
        XCTAssertEqual(decoded.value as? UInt64, UInt64.max)
        XCTAssertEqual(try JSONEncoder().encode(decoded), data)
    }

    func testNestedNumericValuesAndExistingScalars() throws {
        let value = AnyCodable(["counts": [UInt64.max, UInt64(0)] as [Any], "ratio": Float(1.5), "enabled": true, "name": "x", "empty": NSNull()] as [String: Any])
        let data = try JSONEncoder().encode(value)
        let object = try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
        XCTAssertEqual((object["counts"] as? [NSNumber])?.first?.stringValue, "18446744073709551615")
        XCTAssertEqual((object["ratio"] as? NSNumber)?.doubleValue, 1.5)
        XCTAssertEqual(object["enabled"] as? Bool, true)
        XCTAssertEqual(object["name"] as? String, "x")
        XCTAssertTrue(object["empty"] is NSNull)
    }
}
