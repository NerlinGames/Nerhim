use std::os::raw::c_char;
use std::borrow::Cow;
use std::ffi::CString;
use std::ffi::CStr;
use std::io::Cursor;
use std::mem::{self, align_of, size_of};
use ash::util::Align;
use ash::extensions::{khr, ext};
use ash::{vk, Entry, Instance, util};
use ash::vk::SurfaceKHR;
use ash::vk::SurfaceCapabilitiesKHR;
use ash::vk::SurfaceTransformFlagsKHR;
use ash::vk::PresentModeKHR;
use ash::vk::SwapchainKHR;
use ash::vk::Semaphore;
use ash::vk::Queue;
use ash::vk::ShaderModule;
use ash::vk::VertexInputAttributeDescription;
use ash::vk::VertexInputBindingDescription;
use winit::window::Window;
use nalgebra::base::Matrix4;
use nalgebra::{ Point3, Isometry3, Vector3 };
use nalgebra::geometry::Perspective3;

use crate::defaults;

const SHADER_ENTRY_NAME: &str = "main";

pub struct PresentIndex(u32);

pub struct GraphicsSystem 
{
    fullscreen: bool,   // TODO Vulkan needs to resize buffers or it will panic.
    resolution_width: u32,
    resolution_height: u32,

    fov_y: f32,
    pub world_camera: WorldViewProjection,

    //view_widget:
    pub gui_camera: GUIProjection,

    instance: Instance,

    surface: khr::Surface,
    surface_khr: vk::SurfaceKHR,

    pub device: Device,
    pub swapchain: Swapchain,

    debug_utils_msg: vk::DebugUtilsMessengerEXT,
    debug_utils: ext::DebugUtils,
}

impl GraphicsSystem
{
    pub fn new
    (
        window: &Window
    )
    -> GraphicsSystem
    {
        let entry = unsafe { Entry::load().unwrap() };

        let application_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_1);

        let mut extensions: Vec<*const c_char> = ash_window::enumerate_required_extensions(window).unwrap().to_vec();
        extensions.push(ext::DebugUtils::name().as_ptr());

        let layers = Self::debug_layers();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);

        let instance = unsafe { entry.create_instance(&create_info, None).unwrap() };        

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity
            (
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            )
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL)
            .pfn_user_callback(Some(Self::messenger_callback));

        let debug_utils = ext::DebugUtils::new(&entry, &instance);
        let debug_utils_msg = unsafe { debug_utils.create_debug_utils_messenger(&debug_info, None).unwrap() };

        let surface = khr::Surface::new(&entry, &instance);
        let surface_khr = unsafe { ash_window::create_surface(&entry, &instance, window, None).unwrap() };
        
        let device = Device::new(&instance, &surface, &surface_khr);
        Self::check_device_extensions
        (
            &instance,
            &device.physical,
            &[
                // Ray tracing in general.
                khr::AccelerationStructure::name().to_str().unwrap().to_string(),
                khr::RayTracingPipeline::name().to_str().unwrap().to_string(),

                // Needed by VK_KHR_acceleration_structure.
                khr::DeferredHostOperations::name().to_str().unwrap().to_string(),
            ]
        );
        
        let swapchain = Swapchain::new(&instance, &device, &surface, &surface_khr, &window);

        device.submit_setup(&swapchain);

        GraphicsSystem
        {
            fullscreen: defaults::FULLSCREEN,
            resolution_width: defaults::RESOLUTION_WIDTH,
            resolution_height: defaults::RESOLUTION_HEIGHT,
            fov_y: defaults::FOV_Y,
            world_camera: WorldViewProjection::perspective(),           
            gui_camera: GUIProjection::orthographic(),
            instance,
            surface,
            surface_khr,
            debug_utils_msg,
            debug_utils,
            device,
            swapchain
        }
    }    

    pub fn bind_buffer_memory
    <
        T: Copy
    >
    (
        &self,
        data: &Vec<T>,
        flags: vk::BufferUsageFlags,
    )
    -> (vk::Buffer, vk::DeviceMemory)
    {
        unsafe
        {
            let buffer_info = vk::BufferCreateInfo::builder()
                .size(data.len() as u64 * size_of::<T>() as u64)
                .usage(flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            
            let index_buffer = self.device.logical.create_buffer(&buffer_info, None).unwrap();

            let memory_req = self.device.logical.get_buffer_memory_requirements(index_buffer);
            let memory_type_index = self.device.find_memorytype_index
            (
                &memory_req,
                vk::MemoryPropertyFlags::HOST_VISIBLE |
                vk::MemoryPropertyFlags::HOST_COHERENT
            ).unwrap();

            let allocate_info = vk::MemoryAllocateInfo
            {
                allocation_size: memory_req.size,
                memory_type_index,
                ..Default::default()
            };
            let index_memory = self.device.logical.allocate_memory(&allocate_info, None).unwrap();

            let index_ptr = self.device.logical.map_memory(index_memory, 0, memory_req.size, vk::MemoryMapFlags::empty()).unwrap();
            let mut index_slice = Align::new(index_ptr, align_of::<T>() as u64, memory_req.size);
            index_slice.copy_from_slice(&data);
            self.device.logical.unmap_memory(index_memory);

            self.device.logical.bind_buffer_memory(index_buffer, index_memory, 0).unwrap();

            (index_buffer, index_memory)
        }
    }

    fn check_device_extensions
    (
        instance: &ash::Instance,
        physical: &vk::PhysicalDevice,
        required_extensions: &[String]
    )
    {   
        let mut all_supported = true; 
        let extensions = unsafe { instance.enumerate_device_extension_properties(*physical).unwrap() };
        let mut supported: Vec<String> = Vec::new();
        
        for required in required_extensions
        {
            if !extensions.iter().any
            (
                |extension|
                {
                    let extension_string = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()).to_str().unwrap() };

                    if extension_string == required
                    {
                        supported.push(format!("SUPPORTED - {}", required));
                        true
                    }
                    else
                    {                        
                        false
                    }
                }
            )
            {
                supported.push(format!("MISSING - {}", required));
                all_supported = false;
            }
        }

        println!("Used device extensions:");
        for extension in supported
        {
            println!("\t{}", extension);
        }

        if !all_supported
        {
            panic!("System requires all extensions to be supported.");
        }
    }

    fn print_extensions // TODO Needs console command.
    (
        &self
    )
    {    
        let extensions = unsafe { self.instance.enumerate_device_extension_properties(self.device.physical).unwrap() };
        
        println!("List device extensions:");
        for extension in extensions
        {
            //let string = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()).to_str().unwrap() };
            println!("\t{}", unsafe { CStr::from_ptr(extension.extension_name.as_ptr()).to_str().unwrap() });
        }
    }

    pub fn frame_start
    (
        &self
    )
    -> PresentIndex
    {
        unsafe
        {
            let (present_index, _) = self.swapchain.loader.acquire_next_image
            (
                self.swapchain.swapchain,
                u64::MAX,
                self.device.present_semaphore,
                vk::Fence::null()
            ).unwrap();

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.swapchain.renderpass)
                .framebuffer(self.swapchain.framebuffers[present_index as usize])
                .render_area(vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: self.swapchain.resolution })
                .clear_values(&self.device.clear_values);

            self.device.logical.reset_command_buffer(self.device.draw_command_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES).unwrap();
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            self.device.logical.begin_command_buffer(self.device.draw_command_buffer, &command_buffer_begin_info).unwrap();

            self.device.logical.cmd_begin_render_pass(self.device.draw_command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            self.device.logical.cmd_set_viewport(self.device.draw_command_buffer, 0, &self.swapchain.viewports);
            self.device.logical.cmd_set_scissor(self.device.draw_command_buffer, 0, &self.swapchain.scissors);

            PresentIndex(present_index)
        }
    }

    pub fn frame_end
    (
        &mut self,
        index: PresentIndex
    )
    {
        unsafe
        {
            self.device.logical.cmd_end_render_pass(self.device.draw_command_buffer);
            self.device.logical.end_command_buffer(self.device.draw_command_buffer).unwrap();

            let submit_fence = self.device.logical.create_fence(&vk::FenceCreateInfo::default(), None).unwrap();
            let command_buffers = vec![self.device.draw_command_buffer];
            let wait_semaphores = vec![self.device.present_semaphore];
            let signal_semaphores = vec![self.device.rendering_semaphore];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);
            self.device.logical.queue_submit(self.device.queue_present, &[submit_info.build()], submit_fence).unwrap();

            self.device.logical.wait_for_fences(&[submit_fence], true, u64::MAX).unwrap();
            self.device.logical.destroy_fence(submit_fence, None);

            let wait_semaphors = [self.device.rendering_semaphore];
            let swapchains = [self.swapchain.swapchain];
            let image_indices = [index.0];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphors)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            self.swapchain.loader.queue_present(self.device.queue_present, &present_info).unwrap();
        }
    }

    pub fn info
    (
        &self
    )
    -> Info
    {
        let properties = unsafe { self.instance.get_physical_device_properties(self.device.physical) };
        let device_type = match properties.device_type
        {
            vk::PhysicalDeviceType::DISCRETE_GPU => "Dedicated".to_string(),
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated".to_string(),
            invalid => format!("Invalid device type code: {:#?}",  invalid)
        };
        let device = unsafe { CStr::from_ptr(properties.device_name.as_ptr()).to_str().unwrap() };

        Info
        {
            api: "Vulkan".to_string(),
            device: device.to_string(),
            device_type
        }
    }

    unsafe extern "system" fn messenger_callback
    (
        _severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        _type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _user_data: *mut std::os::raw::c_void,
    ) -> vk::Bool32
    {
        let callback_data = *p_callback_data;
        let message_id_number: i32 = callback_data.message_id_number as i32;

        let message = if callback_data.p_message.is_null()
        {
            Cow::from("")
        }
        else
        {
            CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };

        println!();
        println!("ID: {}", &message_id_number.to_string());
        println!("{}", message);

        vk::FALSE
    }

    pub fn destroy
    (
        &mut self
    )
    {
        unsafe
        {
            self.device.logical.device_wait_idle().unwrap();

            self.device.logical.destroy_semaphore(self.device.rendering_semaphore, None);
            self.device.logical.destroy_semaphore(self.device.present_semaphore, None);

            self.device.logical.free_memory(self.swapchain.depth_image_memory, None);
            self.device.logical.destroy_image_view(self.swapchain.depth_image_view, None);
            self.device.logical.destroy_image(self.swapchain.depth_image, None);

            for &image_view in self.swapchain.present_image_views.iter()
            {
                self.device.logical.destroy_image_view(image_view, None);
            }

            for framebuffer in self.swapchain.framebuffers.iter()
            {
                self.device.logical.destroy_framebuffer(*framebuffer, None);
            }

            self.device.logical.destroy_render_pass(self.swapchain.renderpass, None);

            self.device.logical.destroy_command_pool(self.device.pool, None);

            self.swapchain.loader.destroy_swapchain(self.swapchain.swapchain, None);
            self.device.logical.destroy_device(None);
            self.surface.destroy_surface(self.surface_khr, None);

            self.debug_utils.destroy_debug_utils_messenger(self.debug_utils_msg, None);
            self.instance.destroy_instance(None);
        }
    }
    
    #[cfg(debug_assertions)]
    fn debug_layers
    ()
    -> Vec<*const i8> // TODO Does not work under PopOS only Windows.
    {        
        /*let mut layers: Vec<*const c_char>  = Vec::new();// vec![/*CString::new("VK_LAYER_KHRONOS_validation").unwrap(),*/ CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
        layers.push("VK_LAYER_LUNARG_standard_validation".as_ptr() as *const c_char);
        //layers.iter().map(|item| item.as_ptr()).collect::<>();*/
        Vec::new()
    }

    #[cfg(not(debug_assertions))]
    fn debug_layers
    ()
    -> Vec<*const c_char>
    {
        Vec::new()
    }
}

pub struct WorldViewProjection
{
    pub projection: Perspective3<f32>,
    pub transform: Isometry3<f32>,
}

impl WorldViewProjection
{
    fn perspective
    ()    
    -> WorldViewProjection
    {
        WorldViewProjection
        {
            projection: Perspective3::new
            (
                defaults::RESOLUTION_WIDTH as f32 / defaults::RESOLUTION_HEIGHT as f32,
                defaults::FOV_Y,
                defaults::WORLD_Z_NEAR,
                defaults::WORLD_Z_FAR
            ),
            transform: Isometry3::look_at_rh
            (
                &Point3::new(0.0, 0.0, -5.0),
                &Point3::origin(),
                &Vector3::y()
            )
        }
    }
}

pub struct GUIProjection
{
    pub projection: Matrix4<f32>
}

impl GUIProjection
{
    fn orthographic
    ()
    -> GUIProjection
    {
        let width = defaults::RESOLUTION_WIDTH as f32;
        let height = defaults::RESOLUTION_HEIGHT as f32;

        let left = -(width / 2.0);
        let right = width / 2.0;
        let top = -(height / 2.0);
        let bottom = height / 2.0;
        let far = defaults::GUI_Z_FAR;

        let left_right = 2.0 / (right - left);
        let top_bottom = 2.0 / (bottom - top);
        let near_far = 1.0 / far;

        let bound_horizontal = -(right + left) / (right - left);
        let bound_vertical = -(bottom + top) / (bottom - top);

        GUIProjection
        {
            projection: Matrix4::new
            (
                left_right,
                0.0,
                0.0,
                0.0,
                0.0,
                top_bottom,
                0.0,
                0.0,
                0.0,
                0.0,
                near_far,
                0.0,
                bound_horizontal,
                bound_vertical,
                0.0,
                1.0
            )
        }
    }
}

pub struct Info
{
    pub api: String,
    pub device: String,
    pub device_type: String,
}

pub struct Device
{
    clear_values: Vec<vk::ClearValue>,

    pub logical: ash::Device,
    physical: vk::PhysicalDevice,

    memory_props: vk::PhysicalDeviceMemoryProperties,
    queue_family: u32,
    queue_present: Queue,

    pool: vk::CommandPool,
    setup_command_buffer: vk::CommandBuffer,
    pub draw_command_buffer: vk::CommandBuffer, // todo Perhaps move to ECSProcessor or even Shader?

    present_semaphore: Semaphore,
    rendering_semaphore: Semaphore
}

impl Device
{
    pub fn new
    (
        instance: &Instance,
        surface_ld: &khr::Surface,
        surface: &vk::SurfaceKHR
    )
    -> Device
    {
        unsafe
        {
            let clear_values =
            [
                vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 0.0] }},
                vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 }}
            ];

            let physical =
            {
                let devices: Vec<vk::PhysicalDevice> = instance.enumerate_physical_devices().unwrap();

                match devices.len()
                {
                    device_count if device_count == 0 => panic!("No device with Vulkan support found."),
                    _ => *devices
                        .iter()
                        .find
                        (
                            |x|
                            {
                                match instance.get_physical_device_properties(**x).device_type
                                {
                                    vk::PhysicalDeviceType::DISCRETE_GPU => true,
                                    _ => false
                                }
                            }
                        )
                        .unwrap()
                }
            };

            let queue_family =
            {
                let queue_properties = instance.get_physical_device_queue_family_properties(physical);
                queue_properties
                    .iter()
                    .enumerate()
                    .filter_map
                    (
                        |(i, info)|
                        {
                            match info.queue_flags.contains(vk::QueueFlags::GRAPHICS) && surface_ld.get_physical_device_surface_support(physical, i as u32, *surface).unwrap()
                            {
                                true => Some(i),
                                false => None
                            }
                        }
                    )
                    .next()
                    .unwrap() as u32
            };

            let logical =
            {
                let extensions_device = vec![khr::Swapchain::name()];
                let extensions: Vec<*const i8> = extensions_device.iter().map(|x| x.as_ptr()).collect();

                let features = vk::PhysicalDeviceFeatures { shader_clip_distance: 1, ..Default::default() };
                let queue_info = [vk::DeviceQueueCreateInfo::builder().queue_family_index(queue_family).queue_priorities(&[0.5]).build()];
                let device_create_info = vk::DeviceCreateInfo::builder().queue_create_infos(&queue_info).enabled_extension_names(&extensions).enabled_features(&features);

                instance.create_device(physical, &device_create_info, None).unwrap()
            };

            let queue_present = logical.get_device_queue(queue_family, 0);
            let memory_props = instance.get_physical_device_memory_properties(physical);

            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family);
            let pool = logical.create_command_pool(&pool_create_info, None).unwrap();

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(2)
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY);
            let command_buffers = logical.allocate_command_buffers(&command_buffer_allocate_info).unwrap();
            let setup_command_buffer = command_buffers[0];
            let draw_command_buffer = command_buffers[1];

            let semaphore_create_info = vk::SemaphoreCreateInfo::default();
            let present_semaphore = logical.create_semaphore(&semaphore_create_info, None).unwrap();
            let rendering_semaphore = logical.create_semaphore(&semaphore_create_info, None).unwrap();

            Device
            {
                clear_values: clear_values.to_vec(),
                logical,
                physical,
                queue_family,
                queue_present,
                memory_props,
                present_semaphore,
                rendering_semaphore,
                pool,
                draw_command_buffer,
                setup_command_buffer
            }
        }
    }

    fn submit_setup
    (
        &self,
        swapchain: &Swapchain
    )
    {
        unsafe
            {
                self.logical.reset_command_buffer(self.setup_command_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES).unwrap();
                let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                self.logical.begin_command_buffer(self.setup_command_buffer, &command_buffer_begin_info).unwrap();

                let image_subres_range = vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .layer_count(1)
                    .level_count(1)
                    .build();

                let layout_transition_barriers = vk::ImageMemoryBarrier::builder()
                    .image(swapchain.depth_image)
                    .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                    .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .subresource_range(image_subres_range);

                self.logical.cmd_pipeline_barrier(
                    self.setup_command_buffer,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[layout_transition_barriers.build()]);

                self.logical.end_command_buffer(self.setup_command_buffer).unwrap();

                let submit_fence = self.logical.create_fence(&vk::FenceCreateInfo::default(), None).unwrap();
                let command_buffers = vec![self.setup_command_buffer];
                let submit_info = vk::SubmitInfo::builder()
                    .wait_semaphores(&[])
                    .wait_dst_stage_mask(&[])
                    .command_buffers(&command_buffers)
                    .signal_semaphores(&[]);
                self.logical.queue_submit(self.queue_present, &[submit_info.build()], submit_fence).unwrap();

                self.logical.wait_for_fences(&[submit_fence], true, u64::MAX).unwrap();
                self.logical.destroy_fence(submit_fence, None);
            }
    }

    pub fn find_memorytype_index
    (
        &self,
        req: &vk::MemoryRequirements,
        flags: vk::MemoryPropertyFlags
    )
    -> Option<u32>
    {
        // Try to find an exactly matching memory flag.
        let best_suitable_index = Self::find_memorytype_index_f(req, &self.memory_props, flags, |property_flags, flags| { property_flags == flags });

        if best_suitable_index.is_some()
        {
            return best_suitable_index;
        }

        // Otherwise find a memory flag that works.
        Self::find_memorytype_index_f(req, &self.memory_props, flags, |property_flags, flags| { property_flags & flags == flags })
    }

    fn find_memorytype_index_f
    <
        F: Fn(vk::MemoryPropertyFlags, vk::MemoryPropertyFlags) -> bool
    >
    (
        req: &vk::MemoryRequirements,
        memory_props: &vk::PhysicalDeviceMemoryProperties,
        flags: vk::MemoryPropertyFlags,
        f: F
    )
    -> Option<u32>
    {
        let mut memory_type_bits = req.memory_type_bits;
        for (index, ref memory_type) in memory_props.memory_types.iter().enumerate()
        {
            if memory_type_bits & 1 == 1 && f(memory_type.property_flags, flags)
            {
                return Some(index as u32);
            }
            memory_type_bits >>= 1;
        }
        None
    }
}

pub struct Swapchain
{
    framebuffers: Vec<vk::Framebuffer>,
    renderpass: vk::RenderPass,

    loader: khr::Swapchain,
    swapchain: vk::SwapchainKHR,

    format: vk::SurfaceFormatKHR,
    image_count: u32,
    resolution: vk::Extent2D,
    transform: vk::SurfaceTransformFlagsKHR,

    pub viewports: Vec<vk::Viewport>,
    pub scissors: Vec<vk::Rect2D>,

    present_mode: vk::PresentModeKHR,
    present_images: Vec<vk::Image>,
    present_image_views: Vec<vk::ImageView>,
    depth_image: vk::Image,
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory
}

impl Swapchain
{
    fn new
    (
        instance: &Instance,
        device: &Device,
        surface_ld: &khr::Surface,
        surface: &vk::SurfaceKHR,
        window: &Window
    )
    -> Swapchain
    {
        let caps = unsafe { surface_ld.get_physical_device_surface_capabilities(device.physical, *surface).unwrap() };
        let format = Self::format(&device.physical, &surface_ld, &surface);
        let image_count = Self::image_count(&caps);
        let resolution = Self::resolution(&caps, window.inner_size().width, window.inner_size().height);
        let transform = Self::transform(&caps);

        let viewports = [vk::Viewport
        {
            x: 0.0,
            y: 0.0,
            width: resolution.width as f32,
            height: resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: resolution}];

        let present_mode = Self::present_mode(&device.physical, &surface_ld, &surface);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(image_count)
            .image_color_space(format.color_space)
            .image_format(format.format)
            .image_extent(resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let loader = khr::Swapchain::new(instance, &device.logical);
        let swapchain = unsafe { loader.create_swapchain(&swapchain_create_info, None).unwrap() };

        let (present_images, present_image_views) = Self::present_images(&device, &loader, &swapchain, format);
        let (depth_image, depth_image_view, depth_image_memory) = Self::depth_images(&device, &window);

        let renderpass =
        {
            let renderpass_render = vk::AttachmentDescription
            {
                format: format.format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            };

            let renderpass_depth = vk::AttachmentDescription
            {
                format: vk::Format::D16_UNORM,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            };

            let renderpass_attachments = [renderpass_render, renderpass_depth];
            let color_attachment_refs = [vk::AttachmentReference { attachment: 0, layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL }];
            let depth_attachment_ref = vk::AttachmentReference { attachment: 1, layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL };

            let dependencies = [vk::SubpassDependency
            {
                src_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                ..Default::default()
            }];

            let subpasses = [vk::SubpassDescription::builder()
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&depth_attachment_ref)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .build()];

            let renderpass_create_info = vk::RenderPassCreateInfo::builder()
                .attachments(&renderpass_attachments)
                .subpasses(&subpasses)
                .dependencies(&dependencies);

            unsafe { device.logical.create_render_pass(&renderpass_create_info, None).unwrap() }
        };

        let framebuffers: Vec<vk::Framebuffer> = present_image_views
            .iter()
            .map(|&x|
            {
                let framebuffer_attachments = [x, depth_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(renderpass)
                    .attachments(&framebuffer_attachments)
                    .width(resolution.width)
                    .height(resolution.height)
                    .layers(1);

                unsafe { device.logical.create_framebuffer(&frame_buffer_create_info, None).unwrap() }
            })
            .collect();

        Swapchain
        {
            renderpass,
            framebuffers,
            loader,
            swapchain,
            format,
            image_count,
            resolution,
            transform,
            viewports: viewports.to_vec(),
            scissors: scissors.to_vec(),
            present_mode,
            present_images,
            present_image_views,
            depth_image,
            depth_image_view,
            depth_image_memory
        }
    }

    fn present_images
    (
        device: &Device,
        swapchain_ld: &khr::Swapchain,
        swapchain: &SwapchainKHR,
        surface_format: vk::SurfaceFormatKHR
    )
    -> (Vec<vk::Image>, Vec<vk::ImageView>)
    {
        let images = unsafe { swapchain_ld.get_swapchain_images(*swapchain).unwrap() };
        let image_views: Vec<vk::ImageView> = images
            .iter()
            .map
            (
                |&x|
                {
                    let create_view_info = vk::ImageViewCreateInfo::builder()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .components
                        (
                            vk::ComponentMapping 
                            {
                                r: vk::ComponentSwizzle::R,
                                g: vk::ComponentSwizzle::G,
                                b: vk::ComponentSwizzle::B,
                                a: vk::ComponentSwizzle::A,
                            }
                        )
                        .subresource_range
                        (
                            vk::ImageSubresourceRange
                            {
                                aspect_mask: vk::ImageAspectFlags::COLOR,
                                base_mip_level: 0,
                                level_count: 1,
                                base_array_layer: 0,
                                layer_count: 1,
                            }
                        )
                        .image(x);

                    unsafe { device.logical.create_image_view(&create_view_info, None).unwrap() }
                }
            )
            .collect();

        (images, image_views)
    }

    fn depth_images
    (
        device: &Device,
        window: &Window
    )
    -> (vk::Image, vk::ImageView, vk::DeviceMemory)
    {
        let depth_image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::D16_UNORM)
            .extent(vk::Extent3D { width: window.inner_size().width, height: window.inner_size().height, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let depth_image = unsafe { device.logical.create_image(&depth_image_create_info, None).unwrap() };
        let depth_image_memory_req = unsafe { device.logical.get_image_memory_requirements(depth_image) };
        let depth_image_memory_index = device.find_memorytype_index(&depth_image_memory_req, vk::MemoryPropertyFlags::DEVICE_LOCAL).unwrap();

        let depth_image_allocate_info = vk::MemoryAllocateInfo::builder().allocation_size(depth_image_memory_req.size).memory_type_index(depth_image_memory_index);
        let depth_image_memory = unsafe { device.logical.allocate_memory(&depth_image_allocate_info, None).unwrap() };
        unsafe { device.logical.bind_image_memory(depth_image, depth_image_memory, 0).unwrap() };

        let depth_image_view_info = vk::ImageViewCreateInfo::builder()
            .subresource_range
            (
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(depth_image)
            .format(depth_image_create_info.format)
            .view_type(vk::ImageViewType::TYPE_2D);

        let depth_image_view = unsafe{ device.logical.create_image_view(&depth_image_view_info, None).unwrap() };

        (depth_image, depth_image_view, depth_image_memory)
    }

    fn image_count
    (
        capabilities: &vk::SurfaceCapabilitiesKHR
    )
    -> u32
    {
        let desired_count = capabilities.min_image_count + 1;

        match desired_count
        {
            over_max if over_max > capabilities.max_image_count && capabilities.max_image_count > 0 => capabilities.max_image_count,
            _ => desired_count
        }
    }

    fn resolution
    (
        capabilities: &vk::SurfaceCapabilitiesKHR,
        width: u32,
        height: u32
    )
    -> vk::Extent2D
    {
        match capabilities.current_extent.width
        {
            u32::MAX => vk::Extent2D { width, height },
            _ => capabilities.current_extent
        }
    }

    fn format
    (
        device_phys: &vk::PhysicalDevice,
        surface_ld: &khr::Surface,
        surface: &vk::SurfaceKHR
    )
    -> vk::SurfaceFormatKHR
    {
        unsafe
        {
            let surface_formats = surface_ld.get_physical_device_surface_formats(*device_phys, *surface).unwrap();
            surface_formats
                .iter()
                .map(|x| match x.format
                {
                    vk::Format::UNDEFINED => vk::SurfaceFormatKHR
                    {
                        format: vk::Format::B8G8R8_UNORM,
                        color_space: x.color_space,
                    },
                    _ => *x,
                })
                .next()
                .unwrap()
        }
    }

    fn transform
    (
        capabilities: &SurfaceCapabilitiesKHR
    )
    -> SurfaceTransformFlagsKHR
    {
        match capabilities.supported_transforms.contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            true => vk::SurfaceTransformFlagsKHR::IDENTITY,
            false => capabilities.current_transform
        }
    }

    fn present_mode
    (
        device_phys: &vk::PhysicalDevice,
        surface_ld: &khr::Surface,
        surface: &SurfaceKHR
    )
    -> PresentModeKHR
    {
        let present_modes = unsafe { surface_ld.get_physical_device_surface_present_modes(*device_phys, *surface).unwrap() };
        present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::IMMEDIATE)
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }
}

pub struct Shader
{
    pub pipeline: Vec<vk::Pipeline>,
    pub pipeline_layout: vk::PipelineLayout,

    vertex: ShaderModule,
    fragment: ShaderModule
}

impl Shader
{
    pub fn new
    (
        device: &Device,
        swapchain: &Swapchain,    
        vert_spv: Vec<u8>,
        frag_spv: Vec<u8>,
        layout_info: vk::PipelineLayoutCreateInfo,
        vert_in_bind_desc: Vec<VertexInputBindingDescription>,
        vert_in_attr_desc: Vec<VertexInputAttributeDescription>,
        vert_in_asmb_info: vk::PipelineInputAssemblyStateCreateInfo
    )
    -> Shader
    {
        let vertex = Self::create_shader_module(device, vert_spv);
        let fragment = Self::create_shader_module(device, frag_spv);

        let pipeline_layout = unsafe
        {
            device.logical.create_pipeline_layout(&layout_info, None).unwrap()
        };

        let entry_point = CString::new(SHADER_ENTRY_NAME).unwrap();

        let shader_stage_create_infos = 
        [
            vk::PipelineShaderStageCreateInfo
            {
                module: vertex,
                p_name: entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo
            {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: fragment,
                p_name: entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            }
        ];

        let vert_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vert_in_attr_desc)
            .vertex_binding_descriptions(&vert_in_bind_desc);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo
        {
            front_face: vk::FrontFace::CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            ..Default::default()
        };

        let multisampling_state_info = vk::PipelineMultisampleStateCreateInfo
        {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let noop_stencil_state = vk::StencilOpState
        {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };

        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo
        {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state,
            back: noop_stencil_state,
            max_depth_bounds: 1.0,
            ..Default::default()
        };

        let color_blend_attachment_states =
        [
            vk::PipelineColorBlendAttachmentState
            {
                blend_enable: vk::FALSE,
                src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ZERO,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,
                color_write_mask: vk::ColorComponentFlags::RGBA,
            }
        ];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&swapchain.scissors)
            .viewports(&swapchain.viewports);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vert_input_info)
            .input_assembly_state(&vert_in_asmb_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisampling_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(swapchain.renderpass);
        
        let pipeline = unsafe
        {
            device.logical.create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info.build()], None).unwrap()      
        };

        Shader
        {
            pipeline,
            pipeline_layout,
            vertex,
            fragment
        }
    }

    fn create_shader_module
    (
        device: &Device,
        spv_bytes: Vec<u8>
    )
    -> ShaderModule
    {
        let mut seekable_bytes = Cursor::new(spv_bytes.as_slice());
        let binaries = util::read_spv::<Cursor<&[u8]>>(&mut seekable_bytes).unwrap();
        let module_create_info = vk::ShaderModuleCreateInfo::builder().code(&binaries);
        unsafe { device.logical.create_shader_module(&module_create_info, None).unwrap() }
    }

    pub fn destroy
    (
        &self,
        device: &Device
    )
    {
        unsafe
        {
            device.logical.destroy_shader_module(self.vertex, None);
            device.logical.destroy_shader_module(self.fragment, None);

            device.logical.destroy_pipeline(self.pipeline[0], None);
            device.logical.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}