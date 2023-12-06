use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::protocol::errors::ParseError;

/// Measurement units.
///
/// Used in [`crate::protocol::MessageField`].
///
/// # Examples
///
/// ## Construct
///
/// Construct by parsing a string:
///
/// ```rust
/// use mavspec::protocol::Units;
///
/// assert!(matches!(
///     "mAh".parse::<Units>().unwrap(),
///     Units::MilliAmpereHour,
/// ));
/// ```
///
/// Alternatively use [`Units::parse`]:
///
/// ```rust
/// use mavspec::protocol::Units;
///
/// assert!(matches!(
///     Units::parse("m/s").unwrap(),
///     Units::MetresPerSecond
/// ));
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Units {
    /// Time. Second: "s".
    Seconds,
    /// Time. Deci second (second / 10): "ds".
    DeciSeconds,
    /// Time. Centi second (second / 100): "cs".
    CentiSeconds,
    /// Time. Millisecond: "ms".
    MilliSeconds,
    /// Time. Microsecond: "us".
    MicroSeconds,
    /// Time. Nanosecond: "ns".
    NanoSeconds,
    /// Time (frequency). Hertz: "Hz".
    Hertz,
    /// Time (frequency). Megahertz: "MHz".
    MegaHertz,

    /// Distance. Kilometres: "km".
    KiloMetres,
    /// Distance. Decametres: "dam".
    DecaMetres,
    /// Distance. Meter: "m".
    Metres,
    /// Distance (velocity). Metres per second: "m/s".
    MetresPerSecond,
    /// Distance (acceleration). Metres pers second squared: "m/s/s".
    MetresPerSecondSquared,
    /// Distance (velocity). Metres per second: "m/s*5".
    FiveMetresPerSecond,
    /// Distance. Decimetre: "dm".
    DeciMetres,
    /// Distance (velocity). Decimetres per second: "cm/s".
    DeciMetresPerSecond,
    /// Distance. Centimetre: "cm".
    CentiMetres,
    /// Distance (surface). Square centimetre: "cm^2".
    SquareCentiMetres,
    /// Distance (velocity). Centimetres per second: "dm/s".
    CentiMetresPerSecond,
    /// Distance. Millimetre: "mm".
    MilliMetres,
    /// Distance (velocity). Millimetre: "mm/s".
    MilliMetresPerSecond,
    /// Distance (velocity). Millimetres: "mm/h".
    MilliMetresPerHour,

    /// Temperature. Kelvins: "K".
    Kelvins,
    /// Temperature. Degree Celsius: "degC".
    DegreesCelsius,
    /// Temperature. Degree Celsius / 100: "cdegC".
    CentiDegreeCelsius,

    /// Angle. Radians: "rad".
    Radians,
    /// Angle (velocity). Radians per second: "rad/s".
    RadiansPerSecond,
    /// Angle (velocity). Milliradians per second: "mrad/s".
    MilliRadiansPerSecond,
    /// Angle. Degrees: "deg".
    Degrees,
    /// Angle. Half-degrees (degree / 2): "deg/2".
    HalfDegrees,
    /// Angle (velocity). Degrees per second: "deg/s".
    DegreesPerSecond,
    /// Angle. Centi degrees (degree / 100): "cdeg".
    CentiDegrees,
    /// Angle (velocity). Centi degrees (degree / 100) per second: "cdeg/s".
    CentiDegreesPerSecond,
    /// Angle. Degrees / 10^5: "degE5".
    DegreesE5,
    /// Angle. Degrees / 10^7: "degE7".
    DegreesE7,
    /// RotationsPerMinute: "rpm".
    RotationsPerMinute,

    /// Electricity. Volt: "V".
    Volt,
    /// Electricity. Centi volt (volt / 100): "cV".
    CentiVolt,
    /// Electricity. Milli-volt: "mV".
    MilliVolt,
    /// Electricity. Ampere: "A".
    Ampere,
    /// Electricity. Ampere: "Ah".
    AmpereHour,
    /// Electricity. Ampere / 100: "cA".
    CentiAmpere,
    /// Electricity. Milli ampere: "mA".
    MilliAmpere,
    /// Electricity. Milli ampere hour: "mAh".
    MilliAmpereHour,

    /// Magnetism. Milli Tesla: "mT".
    MilliTesla,
    /// Magnetism. Gauss: "gauss".
    Gauss,
    /// Magnetism. Milli-gauss: "mgauss".
    MilliGauss,

    /// Energy. Hecto Joule: "hJ".
    HectoJoule,

    /// Power. Watt: "W".
    Watt,

    /// Force. Milli-G: "mG"
    MilliG,

    /// Mass. Gram: "g"
    Grams,
    /// Mass. Gram: "kg"
    KiloGrams,

    /// Pressure. Pascal: "Pa"
    Pascal,
    /// Pressure. Hectopascal: "hPa"
    HectoPascal,
    /// Pressure. Kilopascal: "kPa"
    KiloPascal,
    /// Pressure. Millibar: "mbar"
    MilliBar,

    /// Ratio. Percent: "%".
    Percent,
    /// Ratio. Decipercent (percent / 10): "d%".
    DeciPercent,
    /// Ratio. Centipercent (percent / 100): "c%".
    CentiPercent,
    /// Ratio. Decibel: "dB".
    DeciBel,
    /// Ratio. Decibel milli-Wats: "dBm".
    DeciBelMilliWats,

    /// Digital. Kibibyte (1024 bytes): "KiB".
    KibiByte,
    /// Digital (throughput). Kibibyte (1024 bytes) per second: "KiB/s".
    KibiBytePerSecond,
    /// Digital. Mebibyte (1024*1024 bytes): "MiB".
    MebiByte,
    /// Digital (throughput). Mebibyte (1024*1024 bytes) per second: "MiB/s".
    MebiBytePerSecond,
    /// Digital. Bytes: "bytes".
    Bytes,
    /// Digital (throughput). Bytes per second: "bytes/s".
    BytesPerSecond,
    /// Digital (throughput). Bits per second: "bits/s".
    BitsPerSecond,
    /// Digital. Pixels: "pix".
    Pixels,
    /// Digital. Decipixels (pixel / 10): "dpix".
    DeciPixels,

    /// Flow. Grams per minute: "g/min".
    GramsPerMinute,
    /// Flow. Cubic centimetres per minute: "cm^3/min".
    CubicCentiMetresPerMinute,

    /// Volume. Cubic centimetres: "cm^3".
    CubicCentiMetres,
    /// Volume. Litres: "l".
    Litres,
}

impl FromStr for Units {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Units::parse(s)
    }
}

impl Units {
    /// Parses field unit from string.
    ///
    /// # Arguments
    ///
    /// * `s` - string representation of unit of measurement.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mavspec::protocol::Units;
    ///
    /// assert!(matches!(
    ///     Units::parse("m/s/s").unwrap(),
    ///     Units::MetresPerSecondSquared
    /// ));
    /// ```
    pub fn parse(s: &str) -> Result<Units, ParseError> {
        let normalized = s.trim();

        let known = match normalized {
            // Time
            "s" => Units::Seconds,
            "ds" => Units::DeciSeconds,
            "cs" => Units::CentiSeconds,
            "ms" => Units::MilliSeconds,
            "us" => Units::MicroSeconds,
            "ns" => Units::NanoSeconds,
            "Hz" => Units::Hertz,
            "MHz" => Units::MegaHertz,
            // Distance
            "km" => Units::KiloMetres,
            "dam" => Units::DecaMetres,
            "m" => Units::Metres,
            "m/s" => Units::MetresPerSecond,
            "m/s/s" => Units::MetresPerSecondSquared,
            "m/s*5" => Units::FiveMetresPerSecond,
            "dm" => Units::DeciMetres,
            "dm/s" => Units::DeciMetresPerSecond,
            "cm" => Units::CentiMetres,
            "cm^2" => Units::SquareCentiMetres,
            "cm/s" => Units::CentiMetresPerSecond,
            "mm" => Units::MilliMetres,
            "mm/s" => Units::MilliMetresPerSecond,
            "mm/h" => Units::MilliMetresPerHour,
            // Temperature
            "K" => Units::Kelvins,
            "degC" => Units::DegreesCelsius,
            "cdegC" => Units::CentiDegreeCelsius,
            // Angles
            "rad" => Units::Radians,
            "rad/s" => Units::RadiansPerSecond,
            "mrad/s" => Units::MilliRadiansPerSecond,
            "deg" => Units::Degrees,
            "deg/2" => Units::HalfDegrees,
            "deg/s" => Units::DegreesPerSecond,
            "cdeg" => Units::CentiDegrees,
            "cdeg/s" => Units::CentiDegreesPerSecond,
            "degE5" => Units::DegreesE5,
            "degE7" => Units::DegreesE7,
            "rpm" => Units::RotationsPerMinute,
            // Electricity
            "V" => Units::Volt,
            "cV" => Units::CentiVolt,
            "mV" => Units::MilliVolt,
            "A" => Units::Ampere,
            "Ah" => Units::AmpereHour,
            "cA" => Units::CentiAmpere,
            "mA" => Units::MilliAmpere,
            "mAh" => Units::MilliAmpereHour,
            // Magnetism
            "mT" => Units::MilliTesla,
            "gauss" => Units::Gauss,
            "mgauss" => Units::MilliGauss,
            // Energy
            "hJ" => Units::HectoJoule,
            // Power
            "W" => Units::Watt,
            // Force
            "mG" => Units::MilliG,
            // Mass
            "g" => Units::Grams,
            "kg" => Units::KiloGrams,
            // Pressure
            "Pa" => Units::Pascal,
            "hPa" => Units::HectoPascal,
            "kPa" => Units::KiloPascal,
            "mbar" => Units::MilliBar,
            // Ratio
            "%" => Units::Percent,
            "d%" => Units::DeciPercent,
            "c%" => Units::CentiPercent,
            "dB" => Units::DeciBel,
            "dBm" => Units::DeciBelMilliWats,
            // Digital
            "KiB" => Units::KibiByte,
            "KiB/s" => Units::KibiBytePerSecond,
            "MiB" => Units::MebiByte,
            "MiB/s" => Units::MebiBytePerSecond,
            "bytes" => Units::Bytes,
            "bytes/s" => Units::BytesPerSecond,
            "bits/s" => Units::BitsPerSecond,
            "pix" => Units::Pixels,
            "dpix" => Units::DeciPixels,
            // Flow
            "g/min" => Units::GramsPerMinute,
            "cm^3/min" => Units::CubicCentiMetresPerMinute,
            // Volume
            "cm^3" => Units::CubicCentiMetres,
            "l" => Units::Litres,
            unknown => return Err(ParseError::UnitsError(unknown.to_string())),
        };

        Ok(known)
    }
}
