struct Params {
    rect:vec4f, clip:vec4f, world_view:vec4f, flags:vec4f,
    grid_strength:f32, gain:f32, gamma:f32, tint:f32,
    stripe_d:f32, row_d:f32, bgr:f32, scale_mode:f32,
}
@group(0) @binding(0) var ui:texture_2d<u32>;
@group(0) @binding(1) var world:texture_2d<u32>;
@group(0) @binding(2) var colors:texture_2d<f32>;
@group(0) @binding(3) var<uniform> P:Params;
@group(0) @binding(4) var coordinates:texture_2d<u32>;
@vertex fn vs_main(@builtin(vertex_index) i:u32)->@builtin(position) vec4f {
    var p=array<vec2f,3>(vec2f(-1.,-1.),vec2f(3.,-1.),vec2f(-1.,3.));
    return vec4f(p[i],0.,1.);
}
fn mixed_color(data:vec4u, raw:u32)->vec3f {
    let front=data.x|(data.y<<8u);
    var mixed=0u;
    for(var shift=0u;shift<=10u;shift+=5u) {
        let lane=select(1u,2u,shift==5u);
        let limit=select(31u,63u,shift==5u);
        let value=min(limit,((((front>>shift)&31u)*lane*data.z+((raw>>shift)&31u)*lane*data.w+8u)>>4u))/lane;
        mixed|=value<<shift;
    }
    return textureLoad(colors,vec2i(i32(mixed&255u),i32(mixed>>8u)),0).rgb;
}
// Neighbouring grid apertures belong to native UI texels, independently of
// the world zoom. Resolve each texel's backdrop at its relocated centre.
fn fetch(x:i32, y:i32)->vec3f {
    let texel=clamp(vec2i(x,y),vec2i(0),vec2i(479,159));
    let pos=P.rect.xy+(vec2f(texel)+vec2f(0.5))*P.rect.zw/vec2f(480.,160.);
    let wp=P.world_view.xy+(pos-P.clip.xy)/P.clip.zw*P.world_view.zw;
    let wc=clamp(vec2i(floor(wp)),vec2i(0),vec2i(textureDimensions(world))-vec2i(1));
    return mixed_color(textureLoad(ui,texel,0),textureLoad(world,wc,0).r);
}
@fragment fn fs_main(@builtin(position) pos:vec4f)->@location(0) vec4f {
    let at=vec2i(pos.xy)-vec2i(P.flags.yz);
    let extent=i32(textureDimensions(coordinates).x);
    if(any(at<vec2i(0)) || any(at>=vec2i(extent))) {discard;}
    let xc=textureLoad(coordinates,vec2i(at.x,0),0).rg;
    let yc=textureLoad(coordinates,vec2i(at.y,1),0).rg;
    if(xc.x==0xffffffffu || yc.x==0xffffffffu) {discard;}
    let data=textureLoad(ui,vec2i(i32(xc.x),i32(yc.x)),0);
    if(data.z==0u && data.w==16u) {discard;}
    let raw=textureLoad(world,vec2i(i32(xc.y),i32(yc.y)),0).r;
    let scale=P.rect.zw/vec2f(480.,160.);
    let p=(pos.xy-P.rect.xy)/scale;
    var rgb=apply_grid(mixed_color(data,raw),p,scale);
    if(P.flags.x>0.5) {
        rgb=select(pow((rgb+vec3f(0.055))/1.055,vec3f(2.4)),rgb/12.92,rgb<=vec3f(0.04045));
    }
    return vec4f(rgb,1.);
}
