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
        // macOS déclaré UNIQUEMENT pour permettre `swift test` sur l'hôte
        // (la slice macOS de l'XCFramework est un artefact de dev). Le produit
        // reste iOS-only — aucune app macOS n'est ciblée.
        .macOS(.v11),
    ],
    products: [
        .library(name: "SyllabifyFr", targets: ["SyllabifyFr"]),
    ],
    targets: [
        // Pre-built binary: the XCFramework produced by build-xcframework.sh
        // (bundles arm64-device + arm64/x86_64-simulator + macOS-universal slices).
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
