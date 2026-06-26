// swift-tools-version:5.9
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Swift Package for syllabify-fr — local consumption.
//
// Workflow :
//   1. Run `swift/scripts/build-xcframework.sh` on macOS to produce
//      `swift/XCFramework/SyllabifyFr.xcframework`.
//   2. Drop the `swift/` folder into your Xcode project as a local Swift
//      Package, or reference it with `.package(path: "...")`.
//   3. `import SyllabifyFr` in your Swift code.

import PackageDescription

let package = Package(
    name: "SyllabifyFr",
    platforms: [
        .iOS(.v13),
    ],
    products: [
        .library(name: "SyllabifyFr", targets: ["SyllabifyFr"]),
    ],
    targets: [
        // Pre-built binary: the XCFramework produced by build-xcframework.sh
        // (bundles arm64-device + arm64-simulator + x86_64-simulator slices).
        .binaryTarget(
            name: "CSyllabifyFr",
            path: "XCFramework/SyllabifyFr.xcframework"
        ),
        // Idiomatic Swift wrappers over the C ABI.
        .target(
            name: "SyllabifyFr",
            dependencies: ["CSyllabifyFr"],
            path: "Sources/SyllabifyFr"
        ),
        .testTarget(
            name: "SyllabifyFrTests",
            dependencies: ["SyllabifyFr"],
            path: "Tests/SyllabifyFrTests"
        ),
    ]
)
