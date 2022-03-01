//! This module contains a wrapper for the raw device handle. 

use std::sync::{Arc, Weak};
use crate::HandleType;

#[cfg(unix)]
type RawHandle = std::os::unix::io::RawFd;
#[cfg(windows)]
type RawHandle = winapi::um::winnt::HANDLE;

#[derive(Debug)]
pub(crate) enum Handle
{
    Arc(Arc<RawHandle>, HandleType),
    Weak(Weak<RawHandle>, HandleType)
}

// TODO: Determine if this is reasonable
unsafe impl Sync for Handle {}
unsafe impl Send for Handle {}

impl Clone for Handle
{
    fn clone(&self) -> Self 
    {
        match self
        {
            Self::Arc(arc, ty) => Self::Weak(Arc::downgrade(arc), *ty),
            Self::Weak(weak, ty) => Self::Weak(weak.clone(), *ty)
        }
    }
}

impl Handle
{
    pub fn raw(&self) -> Option<RawHandle>
    {
        match self
        {
            Handle::Arc(arc, _) => Some(**arc),
            Handle::Weak(weak, _) => weak.upgrade().map(|arc| *arc),
        }
    }

    pub fn handle_type(&self) -> HandleType
    {
        match self
        {
            Handle::Arc(_, inner_ty) | Handle::Weak(_, inner_ty) => *inner_ty,
        }
    }

    pub fn set_type(&mut self, ty: HandleType)
    {
        match self
        {
            Handle::Arc(_, inner_ty) | Handle::Weak(_, inner_ty) => *inner_ty = ty,
        }
    }
}

impl PartialEq for Handle
{
    fn eq(&self, other: &Self) -> bool 
    {
        match self
        {
            Self::Arc(arc, ty) => 
            {
                match other
                {
                    Handle::Arc(arc2, ty2) => ty == ty2 && arc == arc2,
                    Handle::Weak(weak2, ty2) => ty == ty2 && weak2.upgrade().filter(|arc2| arc == arc2).is_some(),
                }
            },
            Self::Weak(weak, ty) =>
            {
                match other
                {
                    Handle::Arc(arc2, ty2) => ty == ty2 && weak.upgrade().filter(|arc| arc == arc2).is_some(),
                    Handle::Weak(weak2, ty2) => ty == ty2 && weak.upgrade().map(|arc| weak2.upgrade().filter(|arc2| arc == *arc2)).is_some(),
                }
            }
        }
    }
}
