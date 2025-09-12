/// Auto-generated bindings for a pre-instantiated version of a
/// component which implements the world `line-follower-robot`.
///
/// This structure is created through [`LineFollowerRobotPre::new`] which
/// takes a [`InstancePre`](wasmtime::component::InstancePre) that
/// has been created through a [`Linker`](wasmtime::component::Linker).
///
/// For more information see [`LineFollowerRobot`] as well.
pub struct LineFollowerRobotPre<T: 'static> {
    instance_pre: wasmtime::component::InstancePre<T>,
    indices: LineFollowerRobotIndices,
}
impl<T: 'static> Clone for LineFollowerRobotPre<T> {
    fn clone(&self) -> Self {
        Self {
            instance_pre: self.instance_pre.clone(),
            indices: self.indices.clone(),
        }
    }
}
impl<_T: 'static> LineFollowerRobotPre<_T> {
    /// Creates a new copy of `LineFollowerRobotPre` bindings which can then
    /// be used to instantiate into a particular store.
    ///
    /// This method may fail if the component behind `instance_pre`
    /// does not have the required exports.
    pub fn new(instance_pre: wasmtime::component::InstancePre<_T>) -> wasmtime::Result<Self> {
        let indices = LineFollowerRobotIndices::new(&instance_pre)?;
        Ok(Self {
            instance_pre,
            indices,
        })
    }
    pub fn engine(&self) -> &wasmtime::Engine {
        self.instance_pre.engine()
    }
    pub fn instance_pre(&self) -> &wasmtime::component::InstancePre<_T> {
        &self.instance_pre
    }
    /// Instantiates a new instance of [`LineFollowerRobot`] within the
    /// `store` provided.
    ///
    /// This function will use `self` as the pre-instantiated
    /// instance to perform instantiation. Afterwards the preloaded
    /// indices in `self` are used to lookup all exports on the
    /// resulting instance.
    pub fn instantiate(
        &self,
        mut store: impl wasmtime::AsContextMut<Data = _T>,
    ) -> wasmtime::Result<LineFollowerRobot> {
        let mut store = store.as_context_mut();
        let instance = self.instance_pre.instantiate(&mut store)?;
        self.indices.load(&mut store, &instance)
    }
}
impl<_T: Send + 'static> LineFollowerRobotPre<_T> {
    /// Same as [`Self::instantiate`], except with `async`.
    pub async fn instantiate_async(
        &self,
        mut store: impl wasmtime::AsContextMut<Data = _T>,
    ) -> wasmtime::Result<LineFollowerRobot> {
        let mut store = store.as_context_mut();
        let instance = self.instance_pre.instantiate_async(&mut store).await?;
        self.indices.load(&mut store, &instance)
    }
}
/// Auto-generated bindings for index of the exports of
/// `line-follower-robot`.
///
/// This is an implementation detail of [`LineFollowerRobotPre`] and can
/// be constructed if needed as well.
///
/// For more information see [`LineFollowerRobot`] as well.
#[derive(Clone)]
pub struct LineFollowerRobotIndices {
    interface0: exports::robot::GuestIndices,
}
/// Auto-generated bindings for an instance a component which
/// implements the world `line-follower-robot`.
///
/// This structure can be created through a number of means
/// depending on your requirements and what you have on hand:
///
/// * The most convenient way is to use
///   [`LineFollowerRobot::instantiate`] which only needs a
///   [`Store`], [`Component`], and [`Linker`].
///
/// * Alternatively you can create a [`LineFollowerRobotPre`] ahead of
///   time with a [`Component`] to front-load string lookups
///   of exports once instead of per-instantiation. This
///   method then uses [`LineFollowerRobotPre::instantiate`] to
///   create a [`LineFollowerRobot`].
///
/// * If you've instantiated the instance yourself already
///   then you can use [`LineFollowerRobot::new`].
///
/// These methods are all equivalent to one another and move
/// around the tradeoff of what work is performed when.
///
/// [`Store`]: wasmtime::Store
/// [`Component`]: wasmtime::component::Component
/// [`Linker`]: wasmtime::component::Linker
pub struct LineFollowerRobot {
    interface0: exports::robot::Guest,
}
const _: () = {
    #[allow(unused_imports)]
    use wasmtime::component::__internal::anyhow;
    impl LineFollowerRobotIndices {
        /// Creates a new copy of `LineFollowerRobotIndices` bindings which can then
        /// be used to instantiate into a particular store.
        ///
        /// This method may fail if the component does not have the
        /// required exports.
        pub fn new<_T>(
            _instance_pre: &wasmtime::component::InstancePre<_T>,
        ) -> wasmtime::Result<Self> {
            let _component = _instance_pre.component();
            let _instance_type = _instance_pre.instance_type();
            let interface0 = exports::robot::GuestIndices::new(_instance_pre)?;
            Ok(LineFollowerRobotIndices { interface0 })
        }
        /// Uses the indices stored in `self` to load an instance
        /// of [`LineFollowerRobot`] from the instance provided.
        ///
        /// Note that at this time this method will additionally
        /// perform type-checks of all exports.
        pub fn load(
            &self,
            mut store: impl wasmtime::AsContextMut,
            instance: &wasmtime::component::Instance,
        ) -> wasmtime::Result<LineFollowerRobot> {
            let _ = &mut store;
            let _instance = instance;
            let interface0 = self.interface0.load(&mut store, &_instance)?;
            Ok(LineFollowerRobot { interface0 })
        }
    }
    impl LineFollowerRobot {
        /// Convenience wrapper around [`LineFollowerRobotPre::new`] and
        /// [`LineFollowerRobotPre::instantiate`].
        pub fn instantiate<_T>(
            store: impl wasmtime::AsContextMut<Data = _T>,
            component: &wasmtime::component::Component,
            linker: &wasmtime::component::Linker<_T>,
        ) -> wasmtime::Result<LineFollowerRobot> {
            let pre = linker.instantiate_pre(component)?;
            LineFollowerRobotPre::new(pre)?.instantiate(store)
        }
        /// Convenience wrapper around [`LineFollowerRobotIndices::new`] and
        /// [`LineFollowerRobotIndices::load`].
        pub fn new(
            mut store: impl wasmtime::AsContextMut,
            instance: &wasmtime::component::Instance,
        ) -> wasmtime::Result<LineFollowerRobot> {
            let indices = LineFollowerRobotIndices::new(&instance.instance_pre(&store))?;
            indices.load(&mut store, instance)
        }
        /// Convenience wrapper around [`LineFollowerRobotPre::new`] and
        /// [`LineFollowerRobotPre::instantiate_async`].
        pub async fn instantiate_async<_T>(
            store: impl wasmtime::AsContextMut<Data = _T>,
            component: &wasmtime::component::Component,
            linker: &wasmtime::component::Linker<_T>,
        ) -> wasmtime::Result<LineFollowerRobot>
        where
            _T: Send,
        {
            let pre = linker.instantiate_pre(component)?;
            LineFollowerRobotPre::new(pre)?
                .instantiate_async(store)
                .await
        }
        pub fn add_to_linker<T, D>(
            linker: &mut wasmtime::component::Linker<T>,
            host_getter: fn(&mut T) -> D::Data<'_>,
        ) -> wasmtime::Result<()>
        where
            D: devices::HostWithStore + diagnostics::HostWithStore,
            for<'a> D::Data<'a>: devices::Host + diagnostics::Host,
            T: 'static,
        {
            devices::add_to_linker::<T, D>(linker, host_getter)?;
            diagnostics::add_to_linker::<T, D>(linker, host_getter)?;
            Ok(())
        }
        pub fn robot(&self) -> &exports::robot::Guest {
            &self.interface0
        }
    }
};
#[allow(clippy::all)]
pub mod devices {
    #[allow(unused_imports)]
    use wasmtime::component::__internal::{Box, anyhow};
    use wasmtime::component::{ComponentType, Lift, Lower};
    /// Errors that can happen when polling devices asynchronously
    #[derive(Debug, ComponentType, Lower, Lift, Clone, Copy, PartialEq, Eq)]
    #[component(enum)]
    #[repr(u8)]
    pub enum PollError {
        /// The provided handle is not valid
        /// (this handle has never been created)
        #[component(name = "invalid-handle")]
        InvalidHandle,
        /// The provided handle has already been consumed
        /// (the detection of this case is best-effort,
        /// otherwise the handle is reported as expired)
        #[component(name = "consumed-handle")]
        ConsumedHandle,
        /// The provided handle is too old and its value has been lost
        /// (more recent values are available and replace the old one)
        #[component(name = "expired-handle")]
        ExpiredHandle,
    }
    const _: () = {
        #[doc(hidden)]
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct LowerPollError {
            tag: wasmtime::ValRaw,
        }
    };
    impl PollError {
        pub fn name(&self) -> &'static str {
            match self {
                PollError::InvalidHandle => "invalid-handle",
                PollError::ConsumedHandle => "consumed-handle",
                PollError::ExpiredHandle => "expired-handle",
            }
        }
    }
    impl core::fmt::Display for PollError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_fmt(format_args!("{0} (error {1})", self.name(), *self as i32))
        }
    }
    impl core::error::Error for PollError {}
    const _: () = {
        if !(1 == <PollError as wasmtime::component::ComponentType>::SIZE32) {
            panic!(
                "assertion failed: 1 == <PollError as wasmtime::component::ComponentType>::SIZE32",
            )
        }
        if !(1 == <PollError as wasmtime::component::ComponentType>::ALIGN32) {
            panic!(
                "assertion failed: 1 == <PollError as wasmtime::component::ComponentType>::ALIGN32",
            )
        }
    };
    /// Time represented as microseconds
    pub type TimeUs = u32;
    const _: () = {
        if !(4 == <TimeUs as wasmtime::component::ComponentType>::SIZE32) {
            panic!("assertion failed: 4 == <TimeUs as wasmtime::component::ComponentType>::SIZE32",)
        }
        if !(4 == <TimeUs as wasmtime::component::ComponentType>::ALIGN32) {
            panic!("assertion failed: 4 == <TimeUs as wasmtime::component::ComponentType>::ALIGN32",)
        }
    };
    /// A handle to a future value
    pub type FutureHandle = u32;
    const _: () = {
        if !(4 == <FutureHandle as wasmtime::component::ComponentType>::SIZE32) {
            panic!(
                "assertion failed: 4 == <FutureHandle as wasmtime::component::ComponentType>::SIZE32",
            )
        }
        if !(4 == <FutureHandle as wasmtime::component::ComponentType>::ALIGN32) {
            panic!(
                "assertion failed: 4 == <FutureHandle as wasmtime::component::ComponentType>::ALIGN32",
            )
        }
    };
    /// The result of a device read operation, composed of 8 bytes.
    /// Valid configurations are:
    /// - up to 8 independent u8 values, used by line sensors and enabled signal
    /// - up to 4 s16 or u16 values (adjacent bytes combined in little endian order),
    ///   used by motor angles, accel, gyro and IMU data
    /// - up to 2 s32 or u32 values (adjacent bytes combined in little endian order),
    ///   used by time values
    /// - values can be unused (unused bytes are set to zero)
    #[derive(Debug, ComponentType, Lower, Lift, Clone, Copy)]
    #[component(record)]
    pub struct DeviceValue {
        #[component(name = "v0")]
        pub v0: u8,
        #[component(name = "v1")]
        pub v1: u8,
        #[component(name = "v2")]
        pub v2: u8,
        #[component(name = "v3")]
        pub v3: u8,
        #[component(name = "v4")]
        pub v4: u8,
        #[component(name = "v5")]
        pub v5: u8,
        #[component(name = "v6")]
        pub v6: u8,
        #[component(name = "v7")]
        pub v7: u8,
    }
    const _: () = {
        #[doc(hidden)]
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct LowerDeviceValue<
            T0: Copy,
            T1: Copy,
            T2: Copy,
            T3: Copy,
            T4: Copy,
            T5: Copy,
            T6: Copy,
            T7: Copy,
        > {
            v0: T0,
            v1: T1,
            v2: T2,
            v3: T3,
            v4: T4,
            v5: T5,
            v6: T6,
            v7: T7,
            _align: [wasmtime::ValRaw; 0],
        }
    };
    const _: () = {
        if !(8 == <DeviceValue as wasmtime::component::ComponentType>::SIZE32) {
            panic!(
                "assertion failed: 8 == <DeviceValue as wasmtime::component::ComponentType>::SIZE32",
            )
        }
        if !(1 == <DeviceValue as wasmtime::component::ComponentType>::ALIGN32) {
            panic!(
                "assertion failed: 1 == <DeviceValue as wasmtime::component::ComponentType>::ALIGN32",
            )
        }
    };
    /// The set of all possible device operations
    #[derive(Debug, ComponentType, Lower, Lift, Clone, Copy)]
    #[component(variant)]
    pub enum DeviceOperation {
        /// Read left bank of line sensors (8 u8 values),
        /// ready every 100us
        #[component(name = "read-line-left")]
        ReadLineLeft,
        /// Read right bank of line sensors (8 u8 values),
        /// ready every 100us
        #[component(name = "read-line-right")]
        ReadLineRight,
        /// Read angle position of motors (2 u16 values),
        /// ready every 100us
        #[component(name = "read-motor-angles")]
        ReadMotorAngles,
        /// Read accelerometer (3 i16 values: front, side and vertical acceleration),
        /// ready every 100us
        #[component(name = "read-accel")]
        ReadAccel,
        /// Read gyroscope (3 i16 values: roll, pitch and yaw angular velocity),
        /// ready every 100us
        #[component(name = "read-gyro")]
        ReadGyro,
        /// Read IMU fused data (3 i16 values: : roll, pitch and yaw angles),
        /// ready every 10_000us
        #[component(name = "read-imu-fused-data")]
        ReadImuFusedData,
        /// Get time elapsed since initialization in microseconds (1 u32 value), always available
        #[component(name = "get-time")]
        GetTime,
        /// Sleep for the provided duration (in microseconds, no output)
        #[component(name = "sleep-for")]
        SleepFor(TimeUs),
        /// Sleep until the provided time instant (in microseconds, no output)
        #[component(name = "sleep-until")]
        SleepUntil(TimeUs),
        /// Get the status of the `enabled` signal (1 u8 value, 0 or 1 with boolean semantics, always available)
        #[component(name = "get-enabled")]
        GetEnabled,
        /// Wait until the status of the `enabled` signal is 1
        #[component(name = "wait-enabled")]
        WaitEnabled,
        /// Wait until the status of the `enabled` signal is 0
        #[component(name = "wait-disabled")]
        WaitDisabled,
    }
    const _: () = {
        #[doc(hidden)]
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct LowerDeviceOperation<T7: Copy, T8: Copy> {
            tag: wasmtime::ValRaw,
            payload: LowerPayloadDeviceOperation<T7, T8>,
        }
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[repr(C)]
        #[derive(Clone, Copy)]
        union LowerPayloadDeviceOperation<T7: Copy, T8: Copy> {
            ReadLineLeft: [wasmtime::ValRaw; 0],
            ReadLineRight: [wasmtime::ValRaw; 0],
            ReadMotorAngles: [wasmtime::ValRaw; 0],
            ReadAccel: [wasmtime::ValRaw; 0],
            ReadGyro: [wasmtime::ValRaw; 0],
            ReadImuFusedData: [wasmtime::ValRaw; 0],
            GetTime: [wasmtime::ValRaw; 0],
            SleepFor: T7,
            SleepUntil: T8,
            GetEnabled: [wasmtime::ValRaw; 0],
            WaitEnabled: [wasmtime::ValRaw; 0],
            WaitDisabled: [wasmtime::ValRaw; 0],
        }
    };
    const _: () = {
        if !(8 == <DeviceOperation as wasmtime::component::ComponentType>::SIZE32) {
            panic!(
                "assertion failed: 8 == <DeviceOperation as wasmtime::component::ComponentType>::SIZE32",
            )
        }
        if !(4 == <DeviceOperation as wasmtime::component::ComponentType>::ALIGN32) {
            panic!(
                "assertion failed: 4 == <DeviceOperation as wasmtime::component::ComponentType>::ALIGN32",
            )
        }
    };
    /// The resulting status of a poll operation
    #[derive(Debug, ComponentType, Lower, Lift, Clone, Copy)]
    #[component(variant)]
    pub enum PollOperationStatus {
        /// The operation is pending (the value is not ready)
        #[component(name = "pending")]
        Pending,
        /// The operation has completed and produced the provided value
        #[component(name = "ready")]
        Ready(DeviceValue),
    }
    const _: () = {
        #[doc(hidden)]
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct LowerPollOperationStatus<T1: Copy> {
            tag: wasmtime::ValRaw,
            payload: LowerPayloadPollOperationStatus<T1>,
        }
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[repr(C)]
        #[derive(Clone, Copy)]
        union LowerPayloadPollOperationStatus<T1: Copy> {
            Pending: [wasmtime::ValRaw; 0],
            Ready: T1,
        }
    };
    const _: () = {
        if !(9 == <PollOperationStatus as wasmtime::component::ComponentType>::SIZE32) {
            panic!(
                "assertion failed: 9 == <PollOperationStatus as wasmtime::component::ComponentType>::SIZE32",
            )
        }
        if !(1 == <PollOperationStatus as wasmtime::component::ComponentType>::ALIGN32) {
            panic!(
                "assertion failed: 1 == <PollOperationStatus as wasmtime::component::ComponentType>::ALIGN32",
            )
        }
    };
    /// The PWM duty cycle associated with the power of one motor driver
    pub type MotorPower = i16;
    const _: () = {
        if !(2 == <MotorPower as wasmtime::component::ComponentType>::SIZE32) {
            panic!(
                "assertion failed: 2 == <MotorPower as wasmtime::component::ComponentType>::SIZE32",
            )
        }
        if !(2 == <MotorPower as wasmtime::component::ComponentType>::ALIGN32) {
            panic!(
                "assertion failed: 2 == <MotorPower as wasmtime::component::ComponentType>::ALIGN32",
            )
        }
    };
    pub trait HostWithStore: wasmtime::component::HasData {}
    impl<_T: ?Sized> HostWithStore for _T where _T: wasmtime::component::HasData {}
    pub trait Host {
        /// Perform a blocking operation (returns the provided value, blocking for the needed time)
        fn device_operation_blocking(&mut self, operation: DeviceOperation) -> DeviceValue;
        /// Initiate and async operation (immediately returns a handle to the future value)
        fn device_operation_async(&mut self, operation: DeviceOperation) -> FutureHandle;
        /// Poll the status of an async operation (returns immediately)
        fn device_poll(&mut self, handle: FutureHandle) -> Result<PollOperationStatus, PollError>;
        /// Wait for an async operation (returns when ready with the result or immediately with an error)
        fn device_wait(&mut self, handle: FutureHandle) -> Result<DeviceValue, PollError>;
        /// Instructs the simulation to forget the handle to an async operation
        /// (is equivalent to dropping the future in Rust)
        fn forget_handle(&mut self, handle: FutureHandle) -> ();
        /// Set the power of both motors
        fn set_motors_power(&mut self, left: MotorPower, right: MotorPower) -> ();
    }
    impl<_T: Host + ?Sized> Host for &mut _T {
        /// Perform a blocking operation (returns the provided value, blocking for the needed time)
        fn device_operation_blocking(&mut self, operation: DeviceOperation) -> DeviceValue {
            Host::device_operation_blocking(*self, operation)
        }
        /// Initiate and async operation (immediately returns a handle to the future value)
        fn device_operation_async(&mut self, operation: DeviceOperation) -> FutureHandle {
            Host::device_operation_async(*self, operation)
        }
        /// Poll the status of an async operation (returns immediately)
        fn device_poll(&mut self, handle: FutureHandle) -> Result<PollOperationStatus, PollError> {
            Host::device_poll(*self, handle)
        }
        /// Wait for an async operation (returns when ready with the result or immediately with an error)
        fn device_wait(&mut self, handle: FutureHandle) -> Result<DeviceValue, PollError> {
            Host::device_wait(*self, handle)
        }
        /// Instructs the simulation to forget the handle to an async operation
        /// (is equivalent to dropping the future in Rust)
        fn forget_handle(&mut self, handle: FutureHandle) -> () {
            Host::forget_handle(*self, handle)
        }
        /// Set the power of both motors
        fn set_motors_power(&mut self, left: MotorPower, right: MotorPower) -> () {
            Host::set_motors_power(*self, left, right)
        }
    }
    pub fn add_to_linker<T, D>(
        linker: &mut wasmtime::component::Linker<T>,
        host_getter: fn(&mut T) -> D::Data<'_>,
    ) -> wasmtime::Result<()>
    where
        D: HostWithStore,
        for<'a> D::Data<'a>: Host,
        T: 'static,
    {
        let mut inst = linker.instance("devices")?;
        inst.func_wrap(
            "device-operation-blocking",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (DeviceOperation,)| {
                let host = &mut host_getter(caller.data_mut());
                let r = Host::device_operation_blocking(host, arg0);
                Ok((r,))
            },
        )?;
        inst.func_wrap(
            "device-operation-async",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (DeviceOperation,)| {
                let host = &mut host_getter(caller.data_mut());
                let r = Host::device_operation_async(host, arg0);
                Ok((r,))
            },
        )?;
        inst.func_wrap(
            "device-poll",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (FutureHandle,)| {
                let host = &mut host_getter(caller.data_mut());
                let r = Host::device_poll(host, arg0);
                Ok((r,))
            },
        )?;
        inst.func_wrap(
            "device-wait",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (FutureHandle,)| {
                let host = &mut host_getter(caller.data_mut());
                let r = Host::device_wait(host, arg0);
                Ok((r,))
            },
        )?;
        inst.func_wrap(
            "forget-handle",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (FutureHandle,)| {
                let host = &mut host_getter(caller.data_mut());
                let r = Host::forget_handle(host, arg0);
                Ok(r)
            },
        )?;
        inst.func_wrap(
            "set-motors-power",
            move |mut caller: wasmtime::StoreContextMut<'_, T>,
                  (arg0, arg1): (MotorPower, MotorPower)| {
                let host = &mut host_getter(caller.data_mut());
                let r = Host::set_motors_power(host, arg0, arg1);
                Ok(r)
            },
        )?;
        Ok(())
    }
}
#[allow(clippy::all)]
pub mod diagnostics {
    #[allow(unused_imports)]
    use wasmtime::component::__internal::{Box, anyhow};
    use wasmtime::component::{ComponentType, Lift, Lower};
    /// a name associated to a value
    #[derive(Debug, ComponentType, Lower, Lift, Clone)]
    #[component(record)]
    pub struct NamedValue {
        /// Value name
        #[component(name = "name")]
        pub name: wasmtime::component::__internal::String,
        /// Named value
        #[component(name = "value")]
        pub value: i32,
    }
    const _: () = {
        #[doc(hidden)]
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct LowerNamedValue<T0: Copy, T1: Copy> {
            name: T0,
            value: T1,
            _align: [wasmtime::ValRaw; 0],
        }
    };
    const _: () = {
        if !(12 == <NamedValue as wasmtime::component::ComponentType>::SIZE32) {
            panic!(
                "assertion failed: 12 == <NamedValue as wasmtime::component::ComponentType>::SIZE32",
            )
        }
        if !(4 == <NamedValue as wasmtime::component::ComponentType>::ALIGN32) {
            panic!(
                "assertion failed: 4 == <NamedValue as wasmtime::component::ComponentType>::ALIGN32",
            )
        }
    };
    /// The kind of values that can be converted in CSV format
    #[derive(Debug, ComponentType, Lower, Lift, Clone)]
    #[component(variant)]
    pub enum ValueKind {
        /// One byte signed value
        #[component(name = "int8")]
        Int8,
        /// Two bytes signed value
        #[component(name = "int16")]
        Int16,
        /// Four bytes signed value
        #[component(name = "int32")]
        Int32,
        /// One byte unsigned value
        #[component(name = "uint8")]
        Uint8,
        /// Two bytes unsigned value
        #[component(name = "uint16")]
        Uint16,
        /// Four bytes unsigned value
        #[component(name = "uint32")]
        Uint32,
        /// A one byte named value (each concrete value is emitted with its name)
        #[component(name = "named")]
        Named(wasmtime::component::__internal::Vec<NamedValue>),
        /// One byte of padding
        #[component(name = "pad8")]
        Pad8,
        /// Two bytes of padding
        #[component(name = "pad16")]
        Pad16,
    }
    const _: () = {
        #[doc(hidden)]
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct LowerValueKind<T6: Copy> {
            tag: wasmtime::ValRaw,
            payload: LowerPayloadValueKind<T6>,
        }
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[repr(C)]
        #[derive(Clone, Copy)]
        union LowerPayloadValueKind<T6: Copy> {
            Int8: [wasmtime::ValRaw; 0],
            Int16: [wasmtime::ValRaw; 0],
            Int32: [wasmtime::ValRaw; 0],
            Uint8: [wasmtime::ValRaw; 0],
            Uint16: [wasmtime::ValRaw; 0],
            Uint32: [wasmtime::ValRaw; 0],
            Named: T6,
            Pad8: [wasmtime::ValRaw; 0],
            Pad16: [wasmtime::ValRaw; 0],
        }
    };
    const _: () = {
        if !(12 == <ValueKind as wasmtime::component::ComponentType>::SIZE32) {
            panic!(
                "assertion failed: 12 == <ValueKind as wasmtime::component::ComponentType>::SIZE32",
            )
        }
        if !(4 == <ValueKind as wasmtime::component::ComponentType>::ALIGN32) {
            panic!(
                "assertion failed: 4 == <ValueKind as wasmtime::component::ComponentType>::ALIGN32",
            )
        }
    };
    /// The description of a column in a CSV file
    #[derive(Debug, ComponentType, Lower, Lift, Clone)]
    #[component(record)]
    pub struct CsvColumn {
        /// Column name
        #[component(name = "name")]
        pub name: wasmtime::component::__internal::String,
        /// Column value kind
        #[component(name = "kind")]
        pub kind: ValueKind,
    }
    const _: () = {
        #[doc(hidden)]
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct LowerCsvColumn<T0: Copy, T1: Copy> {
            name: T0,
            kind: T1,
            _align: [wasmtime::ValRaw; 0],
        }
    };
    const _: () = {
        if !(20 == <CsvColumn as wasmtime::component::ComponentType>::SIZE32) {
            panic!(
                "assertion failed: 20 == <CsvColumn as wasmtime::component::ComponentType>::SIZE32",
            )
        }
        if !(4 == <CsvColumn as wasmtime::component::ComponentType>::ALIGN32) {
            panic!(
                "assertion failed: 4 == <CsvColumn as wasmtime::component::ComponentType>::ALIGN32",
            )
        }
    };
    pub trait HostWithStore: wasmtime::component::HasData {}
    impl<_T: ?Sized> HostWithStore for _T where _T: wasmtime::component::HasData {}
    pub trait Host {
        /// Write a line of text as a log, like writing to a serial line
        /// (each character takes 100 microseconds)
        fn write_line(&mut self, text: wasmtime::component::__internal::String) -> ();
        /// Write a buffer into a file, eventually converting it to CSV
        /// (each byte takes 10 microseconds)
        fn write_file(
            &mut self,
            name: wasmtime::component::__internal::String,
            data: wasmtime::component::__internal::Vec<u8>,
            csv: Option<wasmtime::component::__internal::Vec<CsvColumn>>,
        ) -> ();
    }
    impl<_T: Host + ?Sized> Host for &mut _T {
        /// Write a line of text as a log, like writing to a serial line
        /// (each character takes 100 microseconds)
        fn write_line(&mut self, text: wasmtime::component::__internal::String) -> () {
            Host::write_line(*self, text)
        }
        /// Write a buffer into a file, eventually converting it to CSV
        /// (each byte takes 10 microseconds)
        fn write_file(
            &mut self,
            name: wasmtime::component::__internal::String,
            data: wasmtime::component::__internal::Vec<u8>,
            csv: Option<wasmtime::component::__internal::Vec<CsvColumn>>,
        ) -> () {
            Host::write_file(*self, name, data, csv)
        }
    }
    pub fn add_to_linker<T, D>(
        linker: &mut wasmtime::component::Linker<T>,
        host_getter: fn(&mut T) -> D::Data<'_>,
    ) -> wasmtime::Result<()>
    where
        D: HostWithStore,
        for<'a> D::Data<'a>: Host,
        T: 'static,
    {
        let mut inst = linker.instance("diagnostics")?;
        inst.func_wrap(
            "write-line",
            move |mut caller: wasmtime::StoreContextMut<'_, T>,
                  (arg0,): (wasmtime::component::__internal::String,)| {
                let host = &mut host_getter(caller.data_mut());
                let r = Host::write_line(host, arg0);
                Ok(r)
            },
        )?;
        inst.func_wrap(
            "write-file",
            move |mut caller: wasmtime::StoreContextMut<'_, T>,
                  (arg0, arg1, arg2): (
                wasmtime::component::__internal::String,
                wasmtime::component::__internal::Vec<u8>,
                Option<wasmtime::component::__internal::Vec<CsvColumn>>,
            )| {
                let host = &mut host_getter(caller.data_mut());
                let r = Host::write_file(host, arg0, arg1, arg2);
                Ok(r)
            },
        )?;
        Ok(())
    }
}
pub mod exports {
    #[allow(clippy::all)]
    pub mod robot {
        #[allow(unused_imports)]
        use wasmtime::component::__internal::{Box, anyhow};
        use wasmtime::component::{ComponentType, Lift, Lower};
        /// An RGB color
        #[derive(Debug, Clone, Copy, ComponentType, Lower, Lift)]
        #[component(record)]
        pub struct Color {
            /// Red component
            #[component(name = "r")]
            pub r: u8,
            /// Green component
            #[component(name = "g")]
            pub g: u8,
            /// Blue component
            #[component(name = "b")]
            pub b: u8,
        }
        const _: () = {
            #[doc(hidden)]
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct LowerColor<T0: Copy, T1: Copy, T2: Copy> {
                r: T0,
                g: T1,
                b: T2,
                _align: [wasmtime::ValRaw; 0],
            }
        };
        const _: () = {
            if !(3 == <Color as wasmtime::component::ComponentType>::SIZE32) {
                panic!(
                    "assertion failed: 3 == <Color as wasmtime::component::ComponentType>::SIZE32",
                )
            }
            if !(1 == <Color as wasmtime::component::ComponentType>::ALIGN32) {
                panic!(
                    "assertion failed: 1 == <Color as wasmtime::component::ComponentType>::ALIGN32",
                )
            }
        };
        /// The appearence and build configuration of a robot
        #[derive(Debug, ComponentType, Lower, Lift, Clone)]
        #[component(record)]
        pub struct Configuration {
            /// Robot name
            #[component(name = "name")]
            pub name: wasmtime::component::__internal::String,
            /// Main color
            #[component(name = "color-main")]
            pub color_main: Color,
            /// Secondary color
            #[component(name = "color-secondary")]
            pub color_secondary: Color,
            /// Axle width from wheel to wheel (in mm, 100 to 200)
            #[component(name = "width-axle")]
            pub width_axle: f32,
            /// Length from wheel axles to front (in mm, 100 to 300)
            #[component(name = "length-front")]
            pub length_front: f32,
            /// Length from wheel axles to back (in mm, 10 to 50)
            #[component(name = "length-back")]
            pub length_back: f32,
            /// Clearing from robot to ground at the robot back (in mm, from 1 to wheels radius)
            #[component(name = "clearing-back")]
            pub clearing_back: f32,
            /// Diameter of robot wheels (in mm, from 20 to 40)
            #[component(name = "wheel-diameter")]
            pub wheel_diameter: f32,
            /// Transmission gear ratio numerator (from 1 to 100)
            #[component(name = "gear-ratio-num")]
            pub gear_ratio_num: u32,
            /// Transmission gear ratio denumerator (from 1 to 100)
            #[component(name = "gear-ratio-den")]
            pub gear_ratio_den: u32,
            /// Spacing of line sensors (in mm, from 1 to 15)
            #[component(name = "front-sensors-spacing")]
            pub front_sensors_spacing: f32,
            /// Height of line sensors from the ground (in mm, from 1 to wheels radius)
            #[component(name = "front-sensors-height")]
            pub front_sensors_height: f32,
        }
        const _: () = {
            #[doc(hidden)]
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct LowerConfiguration<
                T0: Copy,
                T1: Copy,
                T2: Copy,
                T3: Copy,
                T4: Copy,
                T5: Copy,
                T6: Copy,
                T7: Copy,
                T8: Copy,
                T9: Copy,
                T10: Copy,
                T11: Copy,
            > {
                name: T0,
                color_main: T1,
                color_secondary: T2,
                width_axle: T3,
                length_front: T4,
                length_back: T5,
                clearing_back: T6,
                wheel_diameter: T7,
                gear_ratio_num: T8,
                gear_ratio_den: T9,
                front_sensors_spacing: T10,
                front_sensors_height: T11,
                _align: [wasmtime::ValRaw; 0],
            }
        };
        const _: () = {
            if !(52 == <Configuration as wasmtime::component::ComponentType>::SIZE32) {
                panic!(
                    "assertion failed: 52 == <Configuration as wasmtime::component::ComponentType>::SIZE32",
                )
            }
            if !(4 == <Configuration as wasmtime::component::ComponentType>::ALIGN32) {
                panic!(
                    "assertion failed: 4 == <Configuration as wasmtime::component::ComponentType>::ALIGN32",
                )
            }
        };
        pub struct Guest {
            setup: wasmtime::component::Func,
            run: wasmtime::component::Func,
        }
        #[derive(Clone)]
        pub struct GuestIndices {
            setup: wasmtime::component::ComponentExportIndex,
            run: wasmtime::component::ComponentExportIndex,
        }
        impl GuestIndices {
            /// Constructor for [`GuestIndices`] which takes a
            /// [`Component`](wasmtime::component::Component) as input and can be executed
            /// before instantiation.
            ///
            /// This constructor can be used to front-load string lookups to find exports
            /// within a component.
            pub fn new<_T>(
                _instance_pre: &wasmtime::component::InstancePre<_T>,
            ) -> wasmtime::Result<GuestIndices> {
                let instance = _instance_pre
                    .component()
                    .get_export_index(None, "robot")
                    .ok_or_else(|| {
                        wasmtime::component::__internal::anyhow::__private::must_use({
                            let error =
                                wasmtime::component::__internal::anyhow::__private::format_err(
                                    format_args!("no exported instance named `robot`"),
                                );
                            error
                        })
                    })?;
                let mut lookup = move |name| {
                    _instance_pre
                        .component()
                        .get_export_index(Some(&instance), name)
                        .ok_or_else(|| {
                            wasmtime::component::__internal::anyhow::__private::must_use({
                                let error =
                                    wasmtime::component::__internal::anyhow::__private::format_err(
                                        format_args!(
                                            "instance export `robot` does not have export `{0}`",
                                            name,
                                        ),
                                    );
                                error
                            })
                        })
                };
                let _ = &mut lookup;
                let setup = lookup("setup")?;
                let run = lookup("run")?;
                Ok(GuestIndices { setup, run })
            }
            pub fn load(
                &self,
                mut store: impl wasmtime::AsContextMut,
                instance: &wasmtime::component::Instance,
            ) -> wasmtime::Result<Guest> {
                let _instance = instance;
                let _instance_pre = _instance.instance_pre(&store);
                let _instance_type = _instance_pre.instance_type();
                let mut store = store.as_context_mut();
                let _ = &mut store;
                let setup = *_instance
                    .get_typed_func::<(), (Configuration,)>(&mut store, &self.setup)?
                    .func();
                let run = *_instance
                    .get_typed_func::<(), ()>(&mut store, &self.run)?
                    .func();
                Ok(Guest { setup, run })
            }
        }
        impl Guest {
            /// Provide robot configuration (is invoked exactly once at the beginning of the simulation)
            pub fn call_setup<S: wasmtime::AsContextMut>(
                &self,
                mut store: S,
            ) -> wasmtime::Result<Configuration> {
                let callee = unsafe {
                    wasmtime::component::TypedFunc::<(), (Configuration,)>::new_unchecked(
                        self.setup,
                    )
                };
                let (ret0,) = callee.call(store.as_context_mut(), ())?;
                callee.post_return(store.as_context_mut())?;
                Ok(ret0)
            }
            /// Robot logic (is invoked exactly once after setup)
            pub fn call_run<S: wasmtime::AsContextMut>(
                &self,
                mut store: S,
            ) -> wasmtime::Result<()> {
                let callee =
                    unsafe { wasmtime::component::TypedFunc::<(), ()>::new_unchecked(self.run) };
                let () = callee.call(store.as_context_mut(), ())?;
                callee.post_return(store.as_context_mut())?;
                Ok(())
            }
        }
    }
}
const _: &str = "package component:line-follower-robot;\n\nworld line-follower-robot {\n    /// Interface for robot devices (provided by simulation host)\n    import devices: interface {\n        /// Errors that can happen when polling devices asynchronously\n        enum poll-error {\n            /// The provided handle is not valid\n            /// (this handle has never been created)\n            invalid-handle,\n            /// The provided handle has already been consumed\n            /// (the detection of this case is best-effort,\n            /// otherwise the handle is reported as expired)\n            consumed-handle,\n            /// The provided handle is too old and its value has been lost\n            /// (more recent values are available and replace the old one)\n            expired-handle,\n        }\n\n        /// Time represented as microseconds\n        type time-us = u32;\n        /// A handle to a future value\n        type future-handle = u32;\n\n        /// The result of a device read operation, composed of 8 bytes.\n        /// Valid configurations are:\n        /// - up to 8 independent u8 values, used by line sensors and enabled signal\n        /// - up to 4 s16 or u16 values (adjacent bytes combined in little endian order),\n        ///   used by motor angles, accel, gyro and IMU data\n        /// - up to 2 s32 or u32 values (adjacent bytes combined in little endian order),\n        ///   used by time values\n        /// - values can be unused (unused bytes are set to zero)\n        record device-value {\n            v0: u8,\n            v1: u8,\n            v2: u8,\n            v3: u8,\n            v4: u8,\n            v5: u8,\n            v6: u8,\n            v7: u8,\n        }\n\n        /// The set of all possible device operations\n        variant device-operation {\n            /// Read left bank of line sensors (8 u8 values),\n            /// ready every 100us\n            read-line-left,\n            /// Read right bank of line sensors (8 u8 values),\n            /// ready every 100us\n            read-line-right,\n            /// Read angle position of motors (2 u16 values),\n            /// ready every 100us\n            read-motor-angles,\n            /// Read accelerometer (3 i16 values: front, side and vertical acceleration),\n            /// ready every 100us\n            read-accel,\n            /// Read gyroscope (3 i16 values: roll, pitch and yaw angular velocity),\n            /// ready every 100us\n            read-gyro,\n            /// Read IMU fused data (3 i16 values: : roll, pitch and yaw angles),\n            /// ready every 10_000us\n            read-imu-fused-data,\n            /// Get time elapsed since initialization in microseconds (1 u32 value), always available\n            get-time,\n            /// Sleep for the provided duration (in microseconds, no output)\n            sleep-for(time-us),\n            /// Sleep until the provided time instant (in microseconds, no output)\n            sleep-until(time-us),\n            /// Get the status of the `enabled` signal (1 u8 value, 0 or 1 with boolean semantics, always available)\n            get-enabled,\n            /// Wait until the status of the `enabled` signal is 1\n            wait-enabled,\n            /// Wait until the status of the `enabled` signal is 0\n            wait-disabled,\n        }\n\n        /// The resulting status of a poll operation\n        variant poll-operation-status {\n            /// The operation is pending (the value is not ready)\n            pending,\n            /// The operation has completed and produced the provided value\n            ready(device-value),\n        }\n\n        /// Perform a blocking operation (returns the provided value, blocking for the needed time)\n        device-operation-blocking: func(operation: device-operation) -> device-value;\n\n        /// Initiate and async operation (immediately returns a handle to the future value)\n        device-operation-async: func(operation: device-operation) -> future-handle;\n\n        /// Poll the status of an async operation (returns immediately)\n        device-poll: func(handle: future-handle) -> result<poll-operation-status, poll-error>;\n\n        /// Instructs the simulation to forget the handle to an async operation\n        /// (is equivalent to dropping the future in Rust)\n        forget-handle: func(handle: future-handle);\n\n        /// The PWM duty cycle associated with the power of one motor driver\n        type motor-power = s16;\n\n        /// Set the power of both motors\n        set-motors-power: func(left: motor-power, right: motor-power);\n    }\n\n    /// Interface for robot diagnostics (provided by simulation host)\n    import diagnostics: interface {\n        /// a name associated to a value\n        record named-value {\n            /// Value name\n            name: string,\n            /// Named value\n            value: s32,\n        }\n\n        /// The kind of values that can be converted in CSV format\n        variant value-kind {\n            /// One byte signed value\n            int8,\n            /// Two bytes signed value\n            int16,\n            /// Four bytes signed value\n            int32,\n            /// One byte unsigned value\n            uint8,\n            /// Two bytes unsigned value\n            uint16,\n            /// Four bytes unsigned value\n            uint32,\n            /// A one byte named value (each concrete value is emitted with its name)\n            named(list<named-value>),\n            /// One byte of padding\n            pad8,\n            /// Two bytes of padding\n            pad16,\n        }\n\n        /// The description of a column in a CSV file\n        record csv-column {\n            /// Column name\n            name: string,\n            /// Column value kind\n            kind: value-kind,\n        }\n\n        /// Write a line of text as a log, like writing to a serial line\n        /// (each character takes 100 microseconds)\n        write-line: func(text: string);\n\n        /// Write a buffer into a file, eventually converting it to CSV\n        /// (each byte takes 10 microseconds)\n        write-file: func(name: string, data: list<u8>, csv: option<list<csv-column>>);\n    }\n\n    /// Interface for robot logic implementation (implemented by robot)\n    export robot: interface {\n        /// An RGB color\n        record color {\n            /// Red component\n            r: u8,\n            /// Green component\n            g: u8,\n            /// Blue component\n            b: u8,\n        }\n\n        /// The appearence and build configuration of a robot\n        record configuration {\n            /// Robot name\n            name: string,\n            /// Main color\n            color-main: color,\n            /// Secondary color\n            color-secondary: color,\n\n            /// Axle width from wheel to wheel (in mm, 100 to 200)\n            width-axle: f32,\n            /// Length from wheel axles to front (in mm, 100 to 300)\n            length-front: f32,\n            /// Length from wheel axles to back (in mm, 10 to 50)\n            length-back: f32,\n            /// Clearing from robot to ground at the robot back (in mm, from 1 to wheels radius)\n            clearing-back: f32,\n\n            /// Diameter of robot wheels (in mm, from 20 to 40)\n            wheel-diameter: f32,\n            /// Transmission gear ratio numerator (from 1 to 100)\n            gear-ratio-num: u32,\n            /// Transmission gear ratio denumerator (from 1 to 100)\n            gear-ratio-den: u32,\n\n            /// Spacing of line sensors (in mm, from 1 to 15)\n            front-sensors-spacing: f32,\n            /// Height of line sensors from the ground (in mm, from 1 to wheels radius)\n            front-sensors-height: f32,\n        }\n\n        /// Provide robot configuration (is invoked exactly once at the beginning of the simulation)\n        setup: func() -> configuration;\n        /// Robot logic (is invoked exactly once after setup)\n        run: func();\n    }\n}\n";
