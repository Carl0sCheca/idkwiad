#[derive(Debug)]
pub struct Render {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub pipeline: String,
    pub index_count: u32,
    pub transform_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
}

impl Render {
    pub fn new<T: bytemuck::Pod>(
        device: &wgpu::Device,
        mesh: (Vec<T>, Vec<u16>),
        pipeline: String,
        transform_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    ) -> Self {
        let vertex_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer Init"),
                contents: bytemuck::cast_slice(&mesh.0),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        let index_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer Init"),
                contents: bytemuck::cast_slice(&mesh.1),
                usage: wgpu::BufferUsages::INDEX,
            },
        );

        Self {
            vertex_buffer,
            index_buffer,
            pipeline,
            index_count: mesh.1.len() as u32,
            transform_buffer,
        }
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        render_pipeline: &'a wgpu::RenderPipeline,
        bind_groups: Option<Vec<(u32, &'a wgpu::BindGroup)>>,
    ) {
        render_pass.set_pipeline(render_pipeline);

        if let Some(groups) = bind_groups {
            groups.iter().for_each(|(id, bind_group)| {
                render_pass.set_bind_group(*id, bind_group, &[]);
            });
        }

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        if let Some(buffer) = self.transform_buffer.as_ref() {
            render_pass.set_vertex_buffer(1, buffer.slice(..));
        }

        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
