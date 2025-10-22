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
    #[derive(Debug, ComponentType, Lower, Lift, Clone, Copy)]
    #[component(record)]
    pub struct FutureHandle {
        /// Unique handle identifier
        #[component(name = "id")]
        pub id: u32,
        /// The time when the future will be ready
        #[component(name = "ready-at")]
        pub ready_at: TimeUs,
    }
    const _: () = {
        #[doc(hidden)]
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct LowerFutureHandle<T0: Copy, T1: Copy> {
            id: T0,
            ready_at: T1,
            _align: [wasmtime::ValRaw; 0],
        }
    };
    const _: () = {
        if !(8 == <FutureHandle as wasmtime::component::ComponentType>::SIZE32) {
            panic!(
                "assertion failed: 8 == <FutureHandle as wasmtime::component::ComponentType>::SIZE32",
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
    ///   used by motor angles, gyro and IMU data
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
        /// ready every period
        #[component(name = "read-line-left")]
        ReadLineLeft,
        /// Read right bank of line sensors (8 u8 values),
        /// ready every period
        #[component(name = "read-line-right")]
        ReadLineRight,
        /// Read angle position of motors (2 u16 values),
        /// ready every period
        #[component(name = "read-motor-angles")]
        ReadMotorAngles,
        /// Read gyroscope (3 i16 values: roll, pitch and yaw angular velocity),
        /// ready every 2 periods
        #[component(name = "read-gyro")]
        ReadGyro,
        /// Read IMU fused data (3 i16 values: : roll, pitch and yaw angles),
        /// ready every 10 periods
        #[component(name = "read-imu-fused-data")]
        ReadImuFusedData,
        /// Get time elapsed since initialization in microseconds (1 u32 value), always available
        #[component(name = "get-time")]
        GetTime,
        /// Get the simulation period microseconds (1 u32 value) and the number of periods computed so far (another u32 value), always available
        #[component(name = "get-period")]
        GetPeriod,
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
            ReadGyro: [wasmtime::ValRaw; 0],
            ReadImuFusedData: [wasmtime::ValRaw; 0],
            GetTime: [wasmtime::ValRaw; 0],
            GetPeriod: [wasmtime::ValRaw; 0],
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
    pub trait Host: Sized {
        /// Perform a device operation (returns immediately the current value if possible, not for sleep or wait operations)
        fn device_operation_immediate(
            &mut self,
            current_fuel: u64,
            operation: DeviceOperation,
        ) -> wasmtime::Result<DeviceValue>;
        /// Perform a blocking operation (returns the provided value, blocking for the needed time)
        fn device_operation_blocking(
            &mut self,
            current_fuel: u64,
            operation: DeviceOperation,
        ) -> wasmtime::Result<DeviceValue>;
        /// Initiate and async operation (immediately returns a handle to the future value)
        fn device_operation_async(
            &mut self,
            current_fuel: u64,
            operation: DeviceOperation,
        ) -> wasmtime::Result<FutureHandle>;
        /// Poll the status of an async operation (returns immediately)
        fn device_poll(
            &mut self,
            current_fuel: u64,
            handle: FutureHandle,
        ) -> wasmtime::Result<PollOperationStatus>;
        /// Signal future values poll loop start and end to the simulation host
        fn poll_loop(&mut self, current_fuel: u64, start: bool) -> wasmtime::Result<()>;
        /// Instructs the simulation to forget the handle to an async operation
        /// (is equivalent to dropping the future in Rust)
        fn forget_handle(&mut self, handle: FutureHandle) -> ();
        /// Set the power of both motors
        fn set_motors_power(
            &mut self,
            current_fuel: u64,
            left: MotorPower,
            right: MotorPower,
        ) -> wasmtime::Result<()>;
    }
    impl<_T: Host + ?Sized> Host for &mut _T {
        /// Perform a device operation (returns immediately the current value if possible, not for sleep or wait operations)
        fn device_operation_immediate(
            &mut self,
            current_fuel: u64,
            operation: DeviceOperation,
        ) -> wasmtime::Result<DeviceValue> {
            Host::device_operation_immediate(*self, current_fuel, operation)
        }
        /// Perform a blocking operation (returns the provided value, blocking for the needed time)
        fn device_operation_blocking(
            &mut self,
            current_fuel: u64,
            operation: DeviceOperation,
        ) -> wasmtime::Result<DeviceValue> {
            Host::device_operation_blocking(*self, current_fuel, operation)
        }
        /// Initiate and async operation (immediately returns a handle to the future value)
        fn device_operation_async(
            &mut self,
            current_fuel: u64,
            operation: DeviceOperation,
        ) -> wasmtime::Result<FutureHandle> {
            Host::device_operation_async(*self, current_fuel, operation)
        }
        /// Poll the status of an async operation (returns immediately)
        fn device_poll(
            &mut self,
            current_fuel: u64,
            handle: FutureHandle,
        ) -> wasmtime::Result<PollOperationStatus> {
            Host::device_poll(*self, current_fuel, handle)
        }
        /// Signal future values poll loop start and end to the simulation host
        fn poll_loop(&mut self, current_fuel: u64, start: bool) -> wasmtime::Result<()> {
            Host::poll_loop(*self, current_fuel, start)
        }
        /// Instructs the simulation to forget the handle to an async operation
        /// (is equivalent to dropping the future in Rust)
        fn forget_handle(&mut self, handle: FutureHandle) -> () {
            Host::forget_handle(*self, handle)
        }
        /// Set the power of both motors
        fn set_motors_power(
            &mut self,
            current_fuel: u64,
            left: MotorPower,
            right: MotorPower,
        ) -> wasmtime::Result<()> {
            Host::set_motors_power(*self, current_fuel, left, right)
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
            "device-operation-immediate",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (DeviceOperation,)| {
                let current_fuel = caller.get_fuel()?;
                let host = &mut host_getter(caller.data_mut());
                let r = Host::device_operation_immediate(host, current_fuel, arg0)?;
                Ok((r,))
            },
        )?;
        inst.func_wrap(
            "device-operation-blocking",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (DeviceOperation,)| {
                let current_fuel = caller.get_fuel()?;
                let host = &mut host_getter(caller.data_mut());
                let r = Host::device_operation_blocking(host, current_fuel, arg0)?;
                Ok((r,))
            },
        )?;
        inst.func_wrap(
            "device-operation-async",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (DeviceOperation,)| {
                let current_fuel = caller.get_fuel()?;
                let host = &mut host_getter(caller.data_mut());
                let r = Host::device_operation_async(host, current_fuel, arg0)?;
                Ok((r,))
            },
        )?;
        inst.func_wrap(
            "device-poll",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (FutureHandle,)| {
                let current_fuel = caller.get_fuel()?;
                let host = &mut host_getter(caller.data_mut());
                let r = Host::device_poll(host, current_fuel, arg0)?;
                Ok((r,))
            },
        )?;
        inst.func_wrap(
            "poll-loop",
            move |mut caller: wasmtime::StoreContextMut<'_, T>, (arg0,): (bool,)| {
                let current_fuel = caller.get_fuel()?;
                let host = &mut host_getter(caller.data_mut());
                Host::poll_loop(host, current_fuel, arg0)?;
                Ok(())
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
                let current_fuel = caller.get_fuel()?;
                let host = &mut host_getter(caller.data_mut());
                let r = Host::set_motors_power(host, current_fuel, arg0, arg1)?;
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
        fn write_line(
            &mut self,
            current_fuel: u64,

            text: wasmtime::component::__internal::String,
        ) -> wasmtime::Result<()>;
        /// Write a buffer into a file, eventually converting it to CSV
        /// (each byte takes 10 microseconds)
        fn write_file(
            &mut self,
            current_fuel: u64,
            name: wasmtime::component::__internal::String,
            data: wasmtime::component::__internal::Vec<u8>,
            csv: Option<wasmtime::component::__internal::Vec<CsvColumn>>,
        ) -> wasmtime::Result<()>;
    }
    impl<_T: Host + ?Sized> Host for &mut _T {
        /// Write a line of text as a log, like writing to a serial line
        /// (each character takes 100 microseconds)
        fn write_line(
            &mut self,
            current_fuel: u64,
            text: wasmtime::component::__internal::String,
        ) -> wasmtime::Result<()> {
            Host::write_line(*self, current_fuel, text)
        }
        /// Write a buffer into a file, eventually converting it to CSV
        /// (each byte takes 10 microseconds)
        fn write_file(
            &mut self,
            current_fuel: u64,
            name: wasmtime::component::__internal::String,
            data: wasmtime::component::__internal::Vec<u8>,
            csv: Option<wasmtime::component::__internal::Vec<CsvColumn>>,
        ) -> wasmtime::Result<()> {
            Host::write_file(*self, current_fuel, name, data, csv)
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
                let current_fuel = caller.get_fuel()?;
                let host = &mut host_getter(caller.data_mut());
                let r = Host::write_line(host, current_fuel, arg0)?;
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
                let current_fuel = caller.get_fuel()?;
                let host = &mut host_getter(caller.data_mut());
                let r = Host::write_file(host, current_fuel, arg0, arg1, arg2)?;
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
