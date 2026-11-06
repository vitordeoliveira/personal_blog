// Draws the picture a shared link to the site shows, 1200x630, in the dark theme's
// colors. It runs on macOS, with no dependencies:
//
//     swift scripts/social_image.swift assets/og.jpg
import AppKit

let width = 1200, height = 630
let output = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "assets/og.jpg"

/// The theme's OKLCH tokens, converted to sRGB.
func color(_ hex: UInt32, _ alpha: CGFloat = 1) -> NSColor {
    NSColor(
        srgbRed: CGFloat((hex >> 16) & 0xff) / 255,
        green: CGFloat((hex >> 8) & 0xff) / 255,
        blue: CGFloat(hex & 0xff) / 255,
        alpha: alpha
    )
}

let background = color(0x0c1015)  // --background, oklch(0.17 0.012 260)
let foreground = color(0xe5e8ec)  // --foreground, oklch(0.93 0.006 260)
let muted = color(0x9499a0)  // --muted-foreground, oklch(0.68 0.012 260)
let primary = color(0xf97a20)  // --primary, oklch(0.716 0.18 49.5)

let rep = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: width, pixelsHigh: height,
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
rep.size = NSSize(width: width, height: height)
NSGraphicsContext.saveGraphicsState()
let context = NSGraphicsContext(bitmapImageRep: rep)!
NSGraphicsContext.current = context
let cg = context.cgContext

// AppKit's origin is the bottom left; `top(y)` converts from the top.
func top(_ y: CGFloat) -> CGFloat { CGFloat(height) - y }

background.setFill()
NSRect(x: 0, y: 0, width: width, height: height).fill()

// An orange glow from the top right corner.
let glow = CGGradient(
    colorsSpace: CGColorSpace(name: CGColorSpace.sRGB),
    colors: [primary.withAlphaComponent(0.22).cgColor, primary.withAlphaComponent(0).cgColor] as CFArray,
    locations: [0, 1])!
cg.drawRadialGradient(
    glow, startCenter: CGPoint(x: 1200, y: top(0)), startRadius: 0,
    endCenter: CGPoint(x: 1200, y: top(0)), endRadius: 760, options: [])

// The favicon's "v." mark, scaled from its 32-unit viewBox to 88px.
let markX: CGFloat = 88, markY: CGFloat = 80, scale: CGFloat = 88.0 / 32.0
func mark(_ x: CGFloat, _ y: CGFloat) -> CGPoint {
    CGPoint(x: markX + x * scale, y: top(markY + y * scale))
}
primary.setFill()
NSBezierPath(
    roundedRect: NSRect(x: markX, y: top(markY + 32 * scale), width: 32 * scale, height: 32 * scale),
    xRadius: 8 * scale, yRadius: 8 * scale
).fill()
let v = NSBezierPath()
v.move(to: mark(8, 10.5))
v.line(to: mark(13.5, 21.5))
v.line(to: mark(19, 10.5))
v.lineWidth = 3.8 * scale
v.lineCapStyle = .round
v.lineJoinStyle = .round
NSColor.white.setStroke()
v.stroke()
NSColor.white.setFill()
let dot = mark(23.6, 20.8)
NSBezierPath(ovalIn: NSRect(x: dot.x - 2.5 * scale, y: dot.y - 2.5 * scale, width: 5 * scale, height: 5 * scale)).fill()

func draw(_ text: NSAttributedString, x: CGFloat, baseline: CGFloat) {
    let line = CTLineCreateWithAttributedString(text)
    cg.textPosition = CGPoint(x: x, y: top(baseline))
    CTLineDraw(line, cg)
}

func font(_ size: CGFloat, _ weight: NSFont.Weight) -> NSFont {
    NSFont.systemFont(ofSize: size, weight: weight)
}

draw(
    NSAttributedString(string: "vitor.ws", attributes: [
        .font: font(42, .bold), .foregroundColor: primary, .kern: -0.6,
    ]),
    x: 88 + 88 + 26, baseline: 80 + 44 + 15)

draw(
    NSAttributedString(string: "Vitor de Oliveira", attributes: [
        .font: font(96, .bold), .foregroundColor: foreground, .kern: -3,
    ]),
    x: 84, baseline: 440)

let tagline = NSMutableAttributedString(
    string: "Software engineer writing about ",
    attributes: [.font: font(40, .regular), .foregroundColor: muted])
for (index, word) in ["Rust", "crypto", "AI"].enumerated() {
    tagline.append(NSAttributedString(string: word, attributes: [
        .font: font(40, .semibold), .foregroundColor: foreground,
    ]))
    let separator = index == 0 ? ", " : index == 1 ? " and " : ""
    tagline.append(NSAttributedString(string: separator, attributes: [
        .font: font(40, .regular), .foregroundColor: muted,
    ]))
}
draw(tagline, x: 88, baseline: 512)

primary.setFill()
NSRect(x: 0, y: 0, width: width, height: 12).fill()

NSGraphicsContext.restoreGraphicsState()
// JPEG keeps the file well under the 300 KB some apps, such as WhatsApp, accept.
try! rep.representation(using: .jpeg, properties: [.compressionFactor: 0.9])!
    .write(to: URL(fileURLWithPath: output))
