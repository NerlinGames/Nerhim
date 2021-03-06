use std::slice;
use rayon::prelude::*;
use std::mem::{self, size_of};
use ash::vk::{self, VertexInputAttributeDescription, ShaderStageFlags, RenderPassBeginInfoBuilder, VertexInputBindingDescription};
use nalgebra::base::Matrix4;
use nalgebra::Isometry3;
use nokden::{Handle, Storage, offset_of, AssetPath};
use nokden::graphics::{Shader, GraphicsSystem};

/// Renders a non-animated mesh at a specific location.
pub struct MeshSystem
{
    shader: Shader,
    pub assets: Storage<MeshAsset>,
    pub instances: Storage<MeshInstance>
}

impl MeshSystem
{
    pub fn new
    (
        graphics: &GraphicsSystem
    )
    -> MeshSystem
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
            .size(2 * size_of::<Matrix4<f32>>() as u32)
            .build();

        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&[push_constant_range])
            .build();
                    
        MeshSystem
        {
            shader: Shader::new
            (
                &graphics.device,
                &graphics.swapchain,
                include_bytes!("shaders/mesh/main.spv_v").to_vec(),
                include_bytes!("shaders/mesh/main.spv_f").to_vec(),
                layout_info,
                vert_in_bind_desc,
                vert_in_attr_desc,
                vert_in_asmb_info
            ),
            assets: Storage::new(),
            instances: Storage::new()
        }
    }

    pub fn update
    (
        &self,
        graphics: &GraphicsSystem,
        world_camera: &Matrix4<f32>
    )
    {
        unsafe
        {
            let dv = &graphics.device;
            dv.logical.cmd_bind_pipeline(dv.draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.shader.pipeline[0]);

            for instance in self.instances.all()
            {
                let instance = instance.read().unwrap();
                let mesh_asset = self.assets.read(&instance.mesh);

                dv.logical.cmd_bind_vertex_buffers(dv.draw_command_buffer, 0, &[mesh_asset.vertex_buffer], &[0]);
                dv.logical.cmd_bind_index_buffer(dv.draw_command_buffer, mesh_asset.index_buffer, 0, vk::IndexType::UINT32); // TODO Needs to use UINT16.

                let mvp = *world_camera;
                let c_u32: *const Matrix4<f32> = &mvp;
                let c_u8: *const u8 = c_u32 as *const _;
                let bytes_camera: &[u8] = slice::from_raw_parts(c_u8, mem::size_of::<Matrix4<f32>>());

                let mvp = instance.transform.to_homogeneous();
                let c_u32: *const Matrix4<f32> = &mvp;
                let c_u8: *const u8 = c_u32 as *const _;
                let bytes_model_position: &[u8] = slice::from_raw_parts(c_u8, mem::size_of::<Matrix4<f32>>());

                dv.logical.cmd_push_constants(dv.draw_command_buffer, self.shader.pipeline_layout, ShaderStageFlags::VERTEX, 0, &[bytes_camera, bytes_model_position].concat());

                dv.logical.cmd_draw_indexed(dv.draw_command_buffer, mesh_asset.index_count, 1, 0, 0, 1);
            }
        }
    }

    pub fn load_asset_obj
    (
        &mut self,
        asset_path: AssetPath,
        graphics: &GraphicsSystem,
    )
    -> Handle<MeshAsset>
    {        
        const VERTEX_PER_FACE: u8 = 3;
        let (models, textures) = tobj::load_obj(&asset_path.0, false).unwrap();
        if models.is_empty() || textures.is_empty()
        {
            panic!
            (
                "Some data is empty: Models {}, Textures {}, for file {}.",
                models.len(),
                textures.len(),
                &asset_path.0.display()
            );
        }
        else
        {
            let mut colors = Vec::new(); // TODO Should not grouped with vertex positions but have its own index for reuse.
            let mut indexes: Vec<u32> = Vec::new();
            let mut positions = Vec::new();
            for model in models
            {
                for surface_index in &model.mesh.indices
                {
                    indexes.push(*surface_index + positions.len() as u32);
                }
    
                for position_index in 0 .. model.mesh.positions.len() / VERTEX_PER_FACE as usize
                {
                    positions.push
                    (
                        [
                            model.mesh.positions[position_index * 3],
                            model.mesh.positions[position_index * 3 + 1],
                            model.mesh.positions[position_index * 3 + 2]
                        ]
                    );
    
                    colors.push
                    (
                        [
                            textures[model.mesh.material_id.unwrap()].diffuse[0],
                            textures[model.mesh.material_id.unwrap()].diffuse[1],
                            textures[model.mesh.material_id.unwrap()].diffuse[2],
                            1.0
                        ]
                    );
                }
            }
    
            let mut input: Vec<VertexInput> = Vec::new();
            for (index, position) in positions.iter().enumerate()
            {
                input.push
                (
                    VertexInput
                    {
                        position:
                        [
                            position[0],
                            position[1],
                            position[2],                        
                        ],
                        color: colors[index]
                    }
                );
            }
    
            let mut highest = 0;
            for vetex_index in &indexes
            {
                if *vetex_index > highest
                {
                    highest = *vetex_index;
                }
            }
    
            if ((input.len() - 1) as u32) < highest
            {
                panic!
                (
                    "The highest vertex index value is not allowed to be higher than the count of inputs minus one. Max Input Index: {}, Highest Detected: {}",
                    input.len() - 1,
                    highest
                );
            }
            
            self.assets.add(MeshAsset::new(&graphics, indexes.clone(), input))
        }        
    }
}

pub struct MeshAsset
{
    index_count: u32,
    index_buffer: vk::Buffer,
    index_memory: vk::DeviceMemory, // TODO Perhaps has no use after new function.
    vertex_buffer: vk::Buffer,
    vertex_memory: vk::DeviceMemory // TODO Perhaps has no use after new function.
}

impl MeshAsset
{
    pub fn new
    (
        graphics: &GraphicsSystem,
        indices: Vec<u32>,
        vertices: Vec<VertexInput>,
    )
    -> MeshAsset
    {
        let (index_buffer, index_memory) = graphics.bind_buffer_memory(&indices, vk::BufferUsageFlags::INDEX_BUFFER);
        let (vertex_buffer, vertex_memory) = graphics.bind_buffer_memory(&vertices, vk::BufferUsageFlags::VERTEX_BUFFER);

        //let accelleration_geometry_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder().
            

        MeshAsset
        {
            index_count: indices.len() as u32,
            index_buffer,
            index_memory,
            vertex_buffer,
            vertex_memory
        }
    }    
}

pub struct MeshInstance
{
    pub transform: Isometry3<f32>,
    pub mesh: Handle<MeshAsset>
}

#[derive(Copy, Clone)]
pub struct VertexInput
{
    pub position: [f32; 3],
    pub color: [f32; 4],
}