// $ odin run .
// $ ffmpeg -i out-odin-%03d.ppm -r 60 out-odin.mp4

package main

import "core:fmt"
import "core:math"
import "core:os"
import "core:strings"

abs :: proc(x: f32) -> f32 {
    return x if x > 0 else -x
}

vec2_yx :: proc(v: [2]f32) -> [2]f32 {
    return {v.y, v.x}
}

vec2_xyyx :: proc(v: [2]f32) -> [4]f32 {
    return {v.x, v.y, v.y, v.x}
}

vec2_cos :: proc(v: [2]f32) -> [2]f32 {
    return {math.cos(v.x), math.cos(v.y)}
}

vec4_sin :: proc(v: [4]f32) -> [4]f32 {
    return {math.sin(v.x), math.sin(v.y), math.sin(v.z), math.sin(v.w)}
}

vec4_tanh :: proc(v: [4]f32) -> [4]f32 {
    return {math.tanh(v.x), math.tanh(v.y), math.tanh(v.z), math.tanh(v.w)}
}

vec4_exp :: proc(v: [4]f32) -> [4]f32 {
    return {math.exp(v.x), math.exp(v.y), math.exp(v.z), math.exp(v.w)}
}

vec2_dot :: proc(a, b: [2]f32) -> f32 {
    return a.x * b.x + a.y * b.y
}

main :: proc() {
    w_ratio: u16 = 16
    h_ratio: u16 = 9
    factor: u16 = 60

    w := w_ratio * factor
    h := h_ratio * factor
    
    max_ts: u16 = 30 // original 240

    for ts in 0..<max_ts {
        // Open output file corresponding to current ts
        output_fp := fmt.tprintf("out-odin-%03d.ppm", ts)
        
        f, err := os.open(output_fp, os.O_WRONLY | os.O_CREATE | os.O_TRUNC, 0o644)
        if err != os.ERROR_NONE {
            fmt.fprintf(os.stderr, "[ERROR] Could not open %s because: %v\n", output_fp, err)
            os.exit(1)
        }
        defer os.close(f)
        
        // Preamble output file
        header := fmt.tprintf("P6\n%d %d\n255\n", w, h)
        os.write_string(f, header)
        
        r := [2]f32{f32(w), f32(h)}
        t := f32(ts) / f32(max_ts) * 2.0 * math.PI
        
        for y in 0..<h {
            for x in 0..<w {
                FC := [2]f32{f32(x), f32(y)}
                p := FC * 2.0
                p = p - r
                p = p / r.y
                
                l := [2]f32{0, 0}
                l = l + (4.0 - 4.0 * abs(0.7 - vec2_dot(p, p)))
                
                v := p * l
                o := [4]f32{0, 0, 0, 0}  // o: output of a single fragment (pixel)
                
                for iy in 1..=8 {
                    tmp0 := vec2_yx(v)
                    tmp0 = tmp0 * f32(iy)
                    tmp0 = tmp0 + [2]f32{0.0, f32(iy)}
                    tmp0 = tmp0 + t
                    tmp0 = vec2_cos(tmp0)
                    tmp0 = tmp0 / f32(iy)
                    tmp0 = tmp0 + 0.7
                    v = v + tmp0

                    tmp1 := vec2_xyyx(v)
                    tmp1 = vec4_sin(tmp1)
                    tmp1 = tmp1 + 1.0
                    tmp1 = tmp1 * abs(v.x - v.y)
                    o = o + tmp1
                }
                
                tmp3 := [4]f32{-1.0, 1.0, 2.0, 0.0} * (-p.y)
                tmp3 = tmp3 + (l.x - 4.0)
                tmp3 = vec4_exp(tmp3)
                tmp3 = tmp3 * 5.0
                o = tmp3 / o
                o = vec4_tanh(o)

                pixel: [3]u8 = {
                    u8(o.x * 255.0),
                    u8(o.y * 255.0),
                    u8(o.z * 255.0),
                }
                os.write(f, pixel[:])
            }
        }
        
        fmt.printf("[INFO] Generated %s (%3d/%3d)\n", output_fp, ts + 1, max_ts)
    }
}
