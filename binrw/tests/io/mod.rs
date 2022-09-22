#[cfg(feature = "std")]
mod bufreader;
#[cfg(not(feature = "std"))]
mod no_std;
mod seek;
mod take_seek;
