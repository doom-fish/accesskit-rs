// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "AccessKitBridge",
    platforms: [.macOS(.v11)],
    products: [
        .library(name: "AccessKitBridge", type: .static, targets: ["AccessKitBridge"]),
    ],
    targets: [
        .target(name: "AccessKitBridge", path: "Sources/AccessKitBridge"),
    ]
)
