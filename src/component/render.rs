#[derive(Debug)]
pub struct Render<'a> {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub pipeline: &'a str,
    pub index_count: u32,
}

impl Render<'_> {
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
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
