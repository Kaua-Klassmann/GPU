@group(0) @binding(0)
var<storage, read_write> matriz: array<u32>;

@group(0) @binding(1)
var<storage, read> width: array<u32>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.y * width[0] + id.x;

    if(index >= arrayLength(&matriz)) {
        return;
    }

    matriz[index] = index % 16;
}
