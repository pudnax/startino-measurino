use std::ffi::{c_char, CStr};

use ash::{
    extensions::{ext::DebugUtils, khr::Surface},
    prelude::*,
    vk::{self, DebugUtilsMessengerEXT},
};
use startino::measure;

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = &unsafe { *p_callback_data };
    let message = unsafe { CStr::from_ptr(callback_data.p_message) }.to_string_lossy();

    println!(
        "{:?}:\n{:?} : {}\n",
        message_severity, message_type, message,
    );

    vk::FALSE
}

fn create_instance(
    entry: &ash::Entry,
) -> VkResult<(ash::Instance, DebugUtilsMessengerEXT, DebugUtils)> {
    let version = match entry.try_enumerate_instance_version()? {
        Some(version) => version,
        None => vk::make_api_version(1, 0, 0, 0),
    };
    let layer_names =
        [unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") }];
    let layers_names_raw: Vec<*const c_char> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();

    let extension_names = vec![DebugUtils::name().as_ptr()];

    let app_info = vk::ApplicationInfo::builder()
        .application_name(unsafe { CStr::from_ptr("Startino".as_ptr() as *const i8) })
        .engine_name(unsafe { CStr::from_ptr("Startino Engine".as_ptr() as *const i8) })
        .engine_version(vk::make_api_version(1, 1, 0, 0))
        .api_version(version);

    let instance_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extension_names);

    let instance = unsafe { entry.create_instance(&instance_info, None) }?;

    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING, // | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));

    let debug_utils_loader = DebugUtils::new(&entry, &instance);
    let debug_callback =
        unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None)? };

    Ok((instance, debug_callback, debug_utils_loader))
}

fn create_device_and_queues(
    entry: &ash::Entry,
    instance: &ash::Instance,
    surface: Option<&vk::SurfaceKHR>,
) -> VkResult<(ash::Device, vk::Queue)> {
    let pdevices = unsafe { instance.enumerate_physical_devices()? };
    let surface_loader = Surface::new(&entry, &instance);
    let (pdevice, queue_family_index) = pdevices
        .iter()
        .find_map(|pdevice| unsafe {
            instance
                .get_physical_device_queue_family_properties(*pdevice)
                .iter()
                .enumerate()
                .find_map(|(index, info)| {
                    let supports_graphic_and_surface =
                        info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                            && if let Some(surface) = surface {
                                surface_loader
                                    .get_physical_device_surface_support(
                                        *pdevice,
                                        index as u32,
                                        *surface,
                                    )
                                    .unwrap()
                            } else {
                                true
                            };
                    if supports_graphic_and_surface {
                        Some((*pdevice, index))
                    } else {
                        None
                    }
                })
        })
        .expect("Couldn't find suitable device.");
    let queue_family_index = queue_family_index as u32;
    // You need to create window beforehhand to enumerate needed extensions
    let device_extension_names_raw = [
        // Swapchain::name().as_ptr(),
    ];

    let features = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        ..Default::default()
    };
    let priorities = [1.0];

    let queue_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index)
        .queue_priorities(&priorities);

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(std::slice::from_ref(&queue_info))
        .enabled_extension_names(&device_extension_names_raw)
        .enabled_features(&features);

    let device = unsafe { instance.create_device(pdevice, &device_create_info, None)? };
    let present_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

    Ok((device, present_queue))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entry = measure!("Loading Vulkan Library.", {
        unsafe { ash::Entry::load()? }
    });

    let (instance, _debug_callback, _debug_utils_loader) =
        measure!("Creating Vulkan Instance.", { create_instance(&entry)? });

    let (_device, _queue) = measure!("Creating Vulkan Device.", {
        create_device_and_queues(&entry, &instance, None)?
    });

    Ok(())
}
