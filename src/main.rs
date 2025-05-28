use std::time;

use wgpu::util::DeviceExt;

const MATRIZ_WIDTH: usize = 1920;
const MATRIZ_HEIGHT: usize = 1080;

fn main() {
    let matriz: Vec<Vec<u32>> = vec![vec![0; MATRIZ_WIDTH]; MATRIZ_HEIGHT];

    let time = time::Instant::now();

    pollster::block_on(run(&matriz));

    println!("Tempo da GPU: {} ms", time.elapsed().as_millis());
}

async fn run(matriz: &Vec<Vec<u32>>) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();

    // Verifica se a GPU suporta compute shaders
    if !adapter
        .get_downlevel_capabilities()
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    {
        panic!("Adapter does not support compute shaders")
    }

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
        })
        .await
        .unwrap();

    // Carrega o shader
    let shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    // Trata os dados para o jeito que a GPU trabalha
    let matriz_linear = matriz.concat();
    let matriz_linear_bytes: &[u8] = bytemuck::cast_slice(&matriz_linear);

    let matriz_width = [matriz[0].len() as u32];
    let matriz_width_bytes: &[u8] = bytemuck::cast_slice(&matriz_width);

    // Cria um buffer para a GPU ler os dados da matriz
    let input_buffer_matriz = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        contents: matriz_linear_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        label: None,
    });

    let input_buffer_width = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        contents: matriz_width_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        label: None,
    });

    // Cria um buffer para a GPU salvar os dados
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: matriz_linear_bytes.len() as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Cria um "contrato" falando que os dados vão estar no group(0) e no binding certo
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
        ],
    });

    // Faz os dados seguirem o contrato
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer_matriz.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: input_buffer_width.as_entire_binding(),
            },
        ],
    });

    // Fala para o shader seguir o "contrato"
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            }),
        ),
        module: &shader_module,
        entry_point: Some("main"),
        cache: None,
        compilation_options: wgpu::PipelineCompilationOptions {
            ..Default::default()
        },
    });

    // Cria um encoder para a GPU
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let mut computer_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
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
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::MaintainBase::Wait).unwrap();

    let data = buffer_slice.get_mapped_range();
    let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();

    for index in 0..20 {
        println!("index {}: {}", index, result[index])
    }
}
