use std::path::Path;

#[derive(Copy, Clone)]
pub struct ResourceId(usize);

pub trait BasicResourceManager {

    fn get_texture_id(&mut self, p: &Path) -> ResourceId;

    fn get_image_dimensions(&self, id: ResourceId) -> (u32, u32);
}

/// Create a BasicResourceManager that does nothing.
/// Usefull when you have either no resources or that
/// you'll stop to parsing.
pub fn create_null_manager() -> NullBasicResourceManager {
    NullBasicResourceManager
}

pub struct NullBasicResourceManager;

impl BasicResourceManager for NullBasicResourceManager {

    fn get_texture_id(&mut self, _: &Path)
        -> ResourceId
    {
        unsafe { new_resource_id(0) }
    }

    fn get_image_dimensions(&self, _: ResourceId) -> (u32, u32) {
        (0, 0)
    }
}


/// Create a new ResourceId instance. Only a BasicResourceManager
/// should care about this function. If you're a ResourceId user,
/// this is probably not the function you're looking for.
///
/// # `unsafe`
///
/// This function is marked unsafe because it should
/// be used only by the creator of a `BasicResourceManager`,
/// or the more specific `ResourceManager` in uil.
pub unsafe fn new_resource_id(id: usize) -> ResourceId {
    ResourceId(id)
}

impl ResourceId {

    /// Returns the underlying id.
    ///
    /// This function is marked unsafe for the same
    /// reasons as `new_resource_id`.
    #[inline]
    pub unsafe fn get(self) -> usize {
        self.0
    }
}
