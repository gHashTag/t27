// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "queen",
    platforms: [
        .macOS(.v13),
        .iOS(.v16)
    ],
    products: [
        .executable(
            name: "queen",
            targets: ["queen"]
        ),
        .library(
            name: "QueenCore",
            targets: ["QueenCore"]
        )
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-async-algorithms", from: "1.0.0"),
        .package(url: "https://github.com/apple/swift-log", from: "1.0.0"),
        .package(url: "https://github.com/apple/swift-collections", from: "1.0.0")
    ],
    targets: [
        .executableTarget(
            name: "queen",
            dependencies: [
                "QueenCore",
                .product(name: "AsyncAlgorithms", package: "swift-async-algorithms"),
                .product(name: "Logging", package: "swift-log")
            ],
            path: "Sources/queen"
        ),
        .target(
            name: "QueenCore",
            dependencies: [
                .product(name: "AsyncAlgorithms", package: "swift-async-algorithms"),
                .product(name: "Logging", package: "swift-log"),
                .product(name: "Collections", package: "swift-collections")
            ],
            path: "Sources/QueenCore"
        ),
        .testTarget(
            name: "QueenTests",
            dependencies: [
                "queen",
                "QueenCore"
            ],
            path: "Tests/QueenTests"
        )
    ]
)