use std::time;

use wgpu::{
    Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferBindingType, BufferDescriptor, BufferUsages, CommandEncoderDescriptor,
    ComputePassDescriptor, ComputePipelineDescriptor, DeviceDescriptor, Instance,
    InstanceDescriptor, MaintainBase, MapMode, PipelineCompilationOptions,
    PipelineLayoutDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
    ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

const MATRIZ_WIDTH: usize = 1920;
const MATRIZ_HEIGHT: usize = 1080;

fn main() {
    let matriz: Vec<Vec<u32>> = vec![vec![0; MATRIZ_WIDTH]; MATRIZ_HEIGHT];

    let time = time::Instant::now();

    pollster::block_on(run(&matriz));

    println!("Tempo da GPU: {} ms", time.elapsed().as_millis());
}

async fn run(matriz: &Vec<Vec<u32>>) {
    let instance = Instance::new(&InstanceDescriptor {
        backends: Backends::PRIMARY,
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor::default())
        .await
        .unwrap();

    // Trata os dados para o jeito que a GPU trabalha
    let matriz_linear = matriz.concat();
    let matriz_linear_bytes: &[u8] = bytemuck::cast_slice(&matriz_linear);

    let matriz_width = [matriz[0].len() as u32];
    let matriz_width_bytes: &[u8] = bytemuck::cast_slice(&matriz_width);

    // Cria um buffer para a GPU ler os dados da matriz
    let input_buffer_matriz = device.create_buffer_init(&BufferInitDescriptor {
        contents: matriz_linear_bytes,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        label: None,
    });

    let input_buffer_width = device.create_buffer_init(&BufferInitDescriptor {
        contents: matriz_width_bytes,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        label: None,
    });

    // Cria um buffer para a GPU salvar os dados
    let output_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("Staging Buffer"),
        size: matriz_linear_bytes.len() as u64,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // CARREGA O SHADER
    let shader_src = include_str!("shader.wgsl");

    let shader_module = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(shader_src.into()),
    });

    // Cria um "contrato" falando que os dados vão estar no group(0) e no binding certo
    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                count: None,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                count: None,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
        ],
    });

    // Faz os dados seguirem o contrato
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: input_buffer_matriz.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: input_buffer_width.as_entire_binding(),
            },
        ],
    });

    // Fala para o shader seguir o "contrato"
    let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: None,
        layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        })),
        module: &shader_module,
        entry_point: Some("main"),
        cache: None,
        compilation_options: PipelineCompilationOptions {
            ..Default::default()
        },
    });

    // Cria um encoder para a GPU
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });

    {
        let mut computer_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
            ..Default::default()
        });

        computer_pass.set_pipeline(&compute_pipeline);
        computer_pass.set_bind_group(0, &bind_group, &[]);

        computer_pass.dispatch_workgroups(
            (matriz_width[0] + 16 - 1) / 16,
            (matriz.len() as u32 + 16 - 1) / 16,
            1,
        );
    }

    encoder.copy_buffer_to_buffer(
        &input_buffer_matriz,
        0,
        &output_buffer,
        0,
        matriz_linear_bytes.len() as u64,
    );

    queue.submit(Some(encoder.finish()));

    // Solicita mapeamento assíncrono
    let buffer_slice = output_buffer.slice(..);
    buffer_slice.map_async(MapMode::Read, |_| {});
    device.poll(MaintainBase::Wait).unwrap();

    let data = buffer_slice.get_mapped_range();
    let result: Vec<i32> = bytemuck::cast_slice(&data).to_vec();

    for index in 0..20 {
        println!("index {}: {}", index, result[index])
    }
}
