//! Differential Drivetrain Control
//!
//! A differential drivetrain (also called a **tank drive** or **skid-steer**) is a robot whose movement is
//! controlled by two independently driven sets of motors on the left and right sides of its
//! chassis. The system operates by adjusting the speed and direction of the left and right
//! motors, enabling the robot to drive straight or execute turns.
//!
//! - If both sets of motors move at the same speed, the robot moves straight.
//! - If one set of motors moves faster than the other, the robot will turn.
//! - If the motors on one side move forward while the other side moves backward, the robot will rotate in place.
//!
//! Differential drivetrains are *nonholonomic*, meaning they cannot strafe laterally.

use core::cell::RefCell;

use vexide::{core::float::Float, devices::smart::motor::MotorError};

use alloc::{rc::Rc, vec::Vec};
use vexide::devices::smart::Motor;

/// A collection of motors mounted in a differential (left/right) configuration.
pub struct Differential {
    left_motors: SharedMotors,
    right_motors: SharedMotors,
}

impl Differential {
    /// Creates a new drivetrain with the provided left/right motors and a tracking system.
    ///
    /// Motors created with the [`drive_motors`] macro may be safely cloned, as they are wrapped
    /// in an [`Arc`] to allow sharing across tasks and between the drivetrain and its tracking
    /// instance if needed.
    pub const fn new(left_motors: SharedMotors, right_motors: SharedMotors) -> Self {
        Self {
            left_motors,
            right_motors,
        }
    }

    /// Sets the voltage of the left and right motors.
    ///
    /// # Errors
    ///
    /// See [`Motor::set_voltage`].
    pub fn set_voltages(&mut self, voltages: impl Into<Voltages>) -> Result<(), MotorError> {
        let voltages = voltages.into();

        for motor in self.left_motors.borrow_mut().iter_mut() {
            motor.set_voltage(voltages.0)?;
        }

        for motor in self.right_motors.borrow_mut().iter_mut() {
            motor.set_voltage(voltages.1)?;
        }

        Ok(())
    }
}

// Internal alias so I don't have to type this shit out a million times.
pub type SharedMotors = Rc<RefCell<Vec<Motor>>>;

/// A macro that creates a set of motors for a [`DifferentialDrivetrain`].
///
/// This macro simplifies the creation of a [`DriveMotors`] collection, which is a sharable, threadsafe
/// wrapper around vexide's non-copyable [`Motor`](vexide::devices::smart::motor::Motor) struct.
///
/// # Examples
///
/// ```
/// let motors = drive_motors![motor1, motor2, motor3];
/// ```
#[macro_export]
macro_rules! shared_motors {
    ( $( $item:expr ),* $(,)?) => {
        {
            use ::core::cell::RefCell;
            use ::alloc::{rc::Rc, vec::Vec};

            let mut temp_vec: Vec<Motor> = Vec::new();

            $(
                temp_vec.push($item);
            )*

            Rc::new(RefCell::new(temp_vec))
        }
    };
}
pub use shared_motors;

/// Left/Right Motor Voltages
///
/// Used as the standard output of a [`Command`] when working with the [`DifferentialDrivetrain`]
/// struct.
///
/// This struct is additionally a [`Command`] in itself, and can be used to run a drivetrain at a
/// fixed voltage.
///
/// [`Command`]: crate::command::Command
/// [`DifferentialDrivetrain`]: crate::differential::DifferentialDrivetrain
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Voltages(pub f64, pub f64);

impl Voltages {
    #[must_use]
    pub const fn from_arcade(linear: f64, angular: f64) -> Self {
        Self(linear + angular, linear - angular)
    }

    /// Returns [`Voltages`] that are less than a provided `max` value while preserving
    /// the ratio between the original left and right values.
    ///
    /// If either motor is over a `max_voltage`, both values will be decresed by the amount
    /// that is "oversaturated" to preserve the ratio between left and right power.
    #[must_use]
    pub fn normalized(&self, max: f64) -> Self {
        let larger_magnitude = self.0.abs().max(self.1.abs()) / max;

        let mut voltages = *self;

        if larger_magnitude > 1.0 {
            voltages.0 /= larger_magnitude;
            voltages.1 /= larger_magnitude;
        }

        voltages
    }

    /// Returns the left voltage.
    #[must_use]
    pub const fn left(&self) -> f64 {
        self.0
    }

    /// Returns the right voltage.
    #[must_use]
    pub const fn right(&self) -> f64 {
        self.1
    }
}

impl From<(f64, f64)> for Voltages {
    fn from(value: (f64, f64)) -> Self {
        Self(value.0, value.1)
    }
}