use std::path::{Path, PathBuf};
use std::slice;
use std::mem::{self, align_of, size_of};
use ash::vk::{self, VertexInputAttributeDescription, ShaderStageFlags, RenderPassBeginInfoBuilder, VertexInputBindingDescription};
use ash::util::Align;
use nalgebra::base::Matrix4;
use nalgebra::{Transform3, Matrix, Isometry3};
use nokden::{Handle, Storage, offset_of, AssetPath};
use nokden::graphics::{WorldViewProjection, Swapchain, ShaderInfo, Shader, GraphicsSystem};
use crate::{NodeSystem, Node};

/// Renders a non-animated mesh at a specific location.
pub struct MeshSystem
{
    shader: Shader,
    pub storage: Storage<Mesh>
}

impl MeshSystem
{
    pub fn new
    (
        graphics: &GraphicsSystem
    )
    -> MeshSystem
    {
        unsafe
        {
            let vert_in_bind_desc = vec!
            [
                VertexInputBindingDescription::builder()
                    .binding(0)
                    .stride(size_of::<VertexInput>() as u32)
                    .input_rate(vk::VertexInputRate::VERTEX)
                    .build()
            ];

            let vert_in_attr_desc = vec!
            [
                VertexInputAttributeDescription::builder()
                    .location(0)
                    .binding(0)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(offset_of!(VertexInput, position) as u32)
                    .build(),
                VertexInputAttributeDescription::builder()
                    .location(1)
                    .binding(0)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .offset(offset_of!(VertexInput, color) as u32)
                    .build()
            ];

            let vert_in_asmb_info = vk::PipelineInputAssemblyStateCreateInfo
            {
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                ..Default::default()
            };

            let push_constant_range = vk::PushConstantRange::builder()
                .stage_flags(ShaderStageFlags::VERTEX)
                .size(size_of::<Matrix4<f32>>() as u32)
                .build();

            let layout_info = vk::PipelineLayoutCreateInfo::builder()
                .push_constant_ranges(&[push_constant_range])
                .build();

            let config = ShaderInfo
            {
                vert_spv_file: include_bytes!("shaders/mesh/main.spv_v").to_vec(),
                frag_spv_file: include_bytes!("shaders/mesh/main.spv_f").to_vec(),
                layout_info,
                vert_in_bind_desc,
                vert_in_attr_desc,
                vert_in_asmb_info
            };

            MeshSystem
            {
                shader: Shader::new
                (
                    &graphics.device,
                    &graphics.swapchain,
                    config
                ),
                storage: Storage::new()
            }
        }
    }

    pub fn update
    (
        &self,
        graphics: &GraphicsSystem,
        nodes: &NodeSystem,
        view_projection: &Matrix4<f32>
    )
    {
        unsafe
        {
            let dv = &graphics.device;
            dv.logical.cmd_bind_pipeline(dv.draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.shader.pipeline[0]);

            for mesh in self.storage.all()
            {
                let mesh = mesh.read().unwrap();

                dv.logical.cmd_bind_vertex_buffers(dv.draw_command_buffer, 0, &[mesh.vertex_buffer], &[0]);
                dv.logical.cmd_bind_index_buffer(dv.draw_command_buffer, mesh.index_buffer, 0, vk::IndexType::UINT32);

                let node = nodes.storage.read(&mesh.node);
                
                let mvp = view_projection * node.isometry.to_homogeneous();
                let c_u32: *const Matrix4<f32> = &mvp;
                let c_u8: *const u8 = c_u32 as *const _;
                let bytes_matrix4: &[u8] = unsafe { slice::from_raw_parts(c_u8, mem::size_of::<Matrix4<f32>>()) };
                dv.logical.cmd_push_constants(dv.draw_command_buffer, self.shader.pipeline_layout, ShaderStageFlags::VERTEX, 0, &bytes_matrix4);

                dv.logical.cmd_draw_indexed(dv.draw_command_buffer, mesh.index_count, 1, 0, 0, 1);
            }
        }
    }

    pub fn load_obj
    (
        &mut self,
        asset_path: AssetPath,
        graphics: &GraphicsSystem,
        node: Handle<Node>
    )
    {        
        let (models, _) = tobj::load_obj(asset_path.0, true).unwrap();
        let indices = models[0].mesh.indices.clone();
        let mut vertices = Vec::<VertexInput>::new();

        const VERTEX_PER_FACE: u8 = 3;
        for i in 0 .. models[0].mesh.positions.len() / VERTEX_PER_FACE as usize
        {
            vertices.push
            (
                VertexInput
                {
                    position: [models[0].mesh.positions[i * 3],
                    models[0].mesh.positions[i * 3 + 1],
                    models[0].mesh.positions[i * 3 + 2]],
                    color: [0.0, 1.0, 0.0, 1.0]
                }
            );
        }

        self.storage.add(Mesh::new(&graphics, indices.clone(), vertices, node));
    }
}

pub struct Mesh
{
    pub node: Handle<Node>,
    index_count: u32,
    index_buffer: vk::Buffer,
    index_memory: vk::DeviceMemory,
    vertex_buffer: vk::Buffer,
    vertex_memory: vk::DeviceMemory
}

impl Mesh
{
    pub fn new
    (
        graphics: &GraphicsSystem,
        indices: Vec<u32>,
        vertices: Vec<VertexInput>,
        node: Handle<Node>
    )
    -> Mesh
    {
        let device = &graphics.device;

        unsafe
        {
            let (index_buffer, index_memory) =
            {
                let buffer_info = vk::BufferCreateInfo::builder()
                    //.size(std::mem::size_of_val(&indices) as u64)
                    .size(indices.len() as u64 * size_of::<u32>() as u64)
                    .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE);
                let index_buffer = device.logical.create_buffer(&buffer_info, None).unwrap();

                let memory_req = device.logical.get_buffer_memory_requirements(index_buffer);
                let memory_type_index = device.find_memorytype_index(&memory_req, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT).unwrap();
                let allocate_info = vk::MemoryAllocateInfo { allocation_size: memory_req.size, memory_type_index, ..Default::default() };
                let index_memory = device.logical.allocate_memory(&allocate_info, None).unwrap();

                let index_ptr = device.logical.map_memory(index_memory, 0, memory_req.size, vk::MemoryMapFlags::empty()).unwrap();
                let mut index_slice = Align::new(index_ptr, align_of::<u32>() as u64, memory_req.size);
                index_slice.copy_from_slice(&indices);
                device.logical.unmap_memory(index_memory);

                device.logical.bind_buffer_memory(index_buffer, index_memory, 0).unwrap();

                (index_buffer, index_memory)
            };

            let (vertex_buffer, vertex_memory) =
            {
                let buffer_info = vk::BufferCreateInfo
                {
                    size: vertices.len() as u64 * size_of::<VertexInput>() as u64,
                    usage: vk::BufferUsageFlags::VERTEX_BUFFER,
                    sharing_mode: vk::SharingMode::EXCLUSIVE,
                    ..Default::default()
                };
                let vertex_buffer = device.logical.create_buffer(&buffer_info, None).unwrap();

                let memory_req = device.logical.get_buffer_memory_requirements(vertex_buffer);
                let memory_type_index = device.find_memorytype_index(&memory_req, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT).unwrap();
                let allocate_info = vk::MemoryAllocateInfo { allocation_size: memory_req.size, memory_type_index, ..Default::default() };
                let vertex_memory = device.logical.allocate_memory(&allocate_info, None).unwrap();

                let vert_ptr = device.logical.map_memory(vertex_memory, 0, memory_req.size, vk::MemoryMapFlags::empty()).unwrap();
                let mut vert_align = Align::new(vert_ptr, align_of::<VertexInput>() as u64, memory_req.size);
                vert_align.copy_from_slice(&vertices);

                device.logical.unmap_memory(vertex_memory);
                device.logical.bind_buffer_memory(vertex_buffer, vertex_memory, 0).unwrap();

                (vertex_buffer, vertex_memory)
            };

            Mesh
            {
                node,
                index_count: indices.len() as u32,
                index_buffer,
                index_memory,
                vertex_buffer,
                vertex_memory
            }
        }
    }    
}

#[derive(Copy, Clone)]
pub struct VertexInput
{
    pub position: [f32; 3],
    pub color: [f32; 4],
}