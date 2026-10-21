//! Type system module — wraps `arrow::datatypes::DataType`.
//!
//! Design notes:
//! - We never re-implement Arrow's type tags; we wrap `ArrowDataType`.
//! - Nullability lives on `Field`, not on `DataType`. Null *values* are handled
//!   by Arrow arrays' validity bitmaps, not by the type system.
//! - `common_supertype` implements a *safe* promotion lattice: the returned
//!   type can represent every value of both inputs without loss.

use arrow::datatypes::DataType as ArrowDataType;

/// A named, possibly-nullable column schema entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

impl Field {
    pub fn new(name: impl Into<String>, data_type: DataType, nullable: bool) -> Self {
        Self { name: name.into(), data_type, nullable }
    }
}

/// Thin wrapper over Arrow's `DataType`. Only supports the primitives listed in
/// `is_supported_primitive` as first-class citizens; other Arrow types may be
/// wrapped but will not participate in promotion.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataType(pub(crate) ArrowDataType);

impl DataType {
    // Constructors for the supported primitive set

    pub fn boolean() -> Self { Self(ArrowDataType::Boolean) }
    pub fn int8()    -> Self { Self(ArrowDataType::Int8) }
    pub fn int16()   -> Self { Self(ArrowDataType::Int16) }
    pub fn int32()   -> Self { Self(ArrowDataType::Int32) }
    pub fn int64()   -> Self { Self(ArrowDataType::Int64) }
    pub fn uint8()   -> Self { Self(ArrowDataType::UInt8) }
    pub fn uint16()  -> Self { Self(ArrowDataType::UInt16) }
    pub fn uint32()  -> Self { Self(ArrowDataType::UInt32) }
    pub fn uint64()  -> Self { Self(ArrowDataType::UInt64) }
    pub fn float32() -> Self { Self(ArrowDataType::Float32) }
    pub fn float64() -> Self { Self(ArrowDataType::Float64) }
    pub fn utf8()    -> Self { Self(ArrowDataType::Utf8) }
    pub fn large_utf8() -> Self { Self(ArrowDataType::LargeUtf8) }
    pub fn binary()  -> Self { Self(ArrowDataType::Binary) }
    pub fn large_binary() -> Self { Self(ArrowDataType::LargeBinary) }
    pub fn fixed_size_binary(n: i32) -> Self { Self(ArrowDataType::FixedSizeBinary(n)) }

    /// Escape hatch: wrap any Arrow type. Callers should expect
    /// `is_supported_primitive() == false` for exotic types.
    pub fn from_arrow(dt: ArrowDataType) -> Self { Self(dt) }

    /// Borrow the underlying Arrow type.
    pub fn as_arrow(&self) -> &ArrowDataType { &self.0 }

    /// Consume into the underlying Arrow type.
    pub fn into_arrow(self) -> ArrowDataType { self.0 }

    // --- Inspection -------------------------------------------------------

    pub fn is_supported_primitive(&self) -> bool {
        matches!(
            self.0,
            ArrowDataType::Boolean
                | ArrowDataType::Int8
                | ArrowDataType::Int16
                | ArrowDataType::Int32
                | ArrowDataType::Int64
                | ArrowDataType::UInt8
                | ArrowDataType::UInt16
                | ArrowDataType::UInt32
                | ArrowDataType::UInt64
                | ArrowDataType::Float32
                | ArrowDataType::Float64
                | ArrowDataType::Utf8
                | ArrowDataType::LargeUtf8
                | ArrowDataType::Binary
                | ArrowDataType::LargeBinary
        )
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self.0, ArrowDataType::Boolean)
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self.0,
            ArrowDataType::Int8
                | ArrowDataType::Int16
                | ArrowDataType::Int32
                | ArrowDataType::Int64
                | ArrowDataType::UInt8
                | ArrowDataType::UInt16
                | ArrowDataType::UInt32
                | ArrowDataType::UInt64
        )
    }

    pub fn is_signed_integer(&self) -> bool {
        matches!(
            self.0,
            ArrowDataType::Int8 | ArrowDataType::Int16 | ArrowDataType::Int32 | ArrowDataType::Int64
        )
    }

    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self.0,
            ArrowDataType::UInt8 | ArrowDataType::UInt16 | ArrowDataType::UInt32 | ArrowDataType::UInt64
        )
    }

    pub fn is_floating(&self) -> bool {
        matches!(self.0, ArrowDataType::Float32 | ArrowDataType::Float64)
    }

    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_floating()
    }

    pub fn is_string(&self) -> bool {
        matches!(self.0, ArrowDataType::Utf8 | ArrowDataType::LargeUtf8)
    }

    pub fn is_binary(&self) -> bool {
        matches!(
            self.0,
            ArrowDataType::Binary | ArrowDataType::LargeBinary | ArrowDataType::FixedSizeBinary(_)
        )
    }

    // --- Compatibility ----------------------------------------------------

    /// Strict structural equality. This is the right check for schema
    /// validation (e.g. "does this file's schema match my expected schema?").
    pub fn is_compatible_with(&self, other: &DataType) -> bool {
        self.0 == other.0
    }
}

/// Bit width of a numeric Arrow type. `None` for non-numeric types.
fn numeric_width(t: &ArrowDataType) -> Option<u8> {
    use ArrowDataType::*;
    match t {
        Int8 | UInt8 => Some(8),
        Int16 | UInt16 => Some(16),
        Int32 | UInt32 | Float32 => Some(32),
        Int64 | UInt64 | Float64 => Some(64),
        _ => None,
    }
}

/// Smallest safe supertype of two numeric types. "Safe" = can represent every
/// value of both inputs without loss of precision or range.
fn numeric_supertype(a: &ArrowDataType, b: &ArrowDataType) -> Option<ArrowDataType> {
    use ArrowDataType::*;

    let aw = numeric_width(a)?;
    let bw = numeric_width(b)?;

    let a_is_float = matches!(a, Float32 | Float64);
    let b_is_float = matches!(b, Float32 | Float64);

    // --- at least one float --------------------------------------------
    if a_is_float || b_is_float {
        // If either side is f64, the result must be f64.
        if matches!(a, Float64) || matches!(b, Float64) {
            return Some(Float64);
        }
        // Both float-ish here means one is Float32 and the other is int/f32.
        // f32 has a 24-bit mantissa: it can represent all i16/u16 exactly, but
        // not all i32/u32. So widen to f64 for 32/64-bit integers.
        // let other = if a_is_float { b } else { a };
        let other_w = if a_is_float { bw } else { aw };
        return match other_w {
            8 | 16 => Some(Float32),
            _ => Some(Float64), // 32 or 64-bit integer, or the other is Float32
        };
    }

    // --- both integers --------------------------------------------------
    let a_signed = matches!(a, Int8 | Int16 | Int32 | Int64);
    let b_signed = matches!(b, Int8 | Int16 | Int32 | Int64);

    match (a_signed, b_signed) {
        (true, true) => Some(match aw.max(bw) {
            8 => Int8,
            16 => Int16,
            32 => Int32,
            _ => Int64,
        }),
        (false, false) => Some(match aw.max(bw) {
            8 => UInt8,
            16 => UInt16,
            32 => UInt32,
            _ => UInt64,
        }),
        _ => {
            // Mixed signed/unsigned.
            let (w_signed, w_unsigned) = if a_signed { (aw, bw) } else { (bw, aw) };
            if w_signed > w_unsigned {
                // Signed side is already wider → its range subsumes the
                // unsigned side (until we hit Int64/UInt64, handled below).
                return match w_signed {
                    16 => Some(Int16),
                    32 => Some(Int32),
                    64 => {
                        // Int64 vs UInt8/16/32: Int64's range covers all of
                        // them. UInt64 is only a problem when *it* is the
                        // unsigned operand — which it isn't here.
                        Some(Int64)
                    }
                    _ => None,
                };
            }
            // Same width, or signed is narrower → widen one step past the
            // unsigned operand's width. UInt64 + any signed → unsafe.
            match w_unsigned {
                8 => Some(Int16),
                16 => Some(Int32),
                32 => Some(Int64),
                _ => None, // UInt64 mixed with a signed int: no safe supertype
            }
        }
    }
}

/// Compute a safe common supertype of `a` and `b`.
///
/// Returns `None` when no lossless common type exists (e.g. `Int64` ∪ `UInt64`,
/// `Utf8` ∪ `Binary`, `Boolean` ∪ `Int32`).
pub fn common_supertype(a: &DataType, b: &DataType) -> Option<DataType> {
    use ArrowDataType::*;
    let (a_ref, b_ref) = (&a.0, &b.0);

    if a_ref == b_ref {
        return Some(DataType(a_ref.clone()));
    }

    let result = match (a_ref, b_ref) {
        // String family: large wins.
        (Utf8, LargeUtf8) | (LargeUtf8, Utf8) => Some(LargeUtf8),

        // Binary family: large wins; variable wins over fixed (a fixed-size
        // binary value is trivially a variable-length binary value).
        (Binary, LargeBinary) | (LargeBinary, Binary) => Some(LargeBinary),
        (FixedSizeBinary(_), Binary) | (Binary, FixedSizeBinary(_)) => Some(Binary),
        (FixedSizeBinary(_), LargeBinary) | (LargeBinary, FixedSizeBinary(_)) => Some(LargeBinary),
        // FixedSizeBinary(n) ∪ FixedSizeBinary(m), n != m → no safe supertype
        (FixedSizeBinary(_), FixedSizeBinary(_)) => None,

        // Everything else: try numeric promotion.
        _ => numeric_supertype(a_ref, b_ref),
    };

    result.map(DataType)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ArrowDataType::*;

    fn sup(a: ArrowDataType, b: ArrowDataType) -> Option<ArrowDataType> {
        common_supertype(&DataType(a), &DataType(b)).map(|d| d.0)
    }

    #[test]
    fn same_type_is_idempotent() {
        for t in [
            Boolean, Int8, Int16, Int32, Int64,
            UInt8, UInt16, UInt32, UInt64,
            Float32, Float64, Utf8, LargeUtf8, Binary, LargeBinary,
        ] {
            assert_eq!(sup(t.clone(), t.clone()), Some(t));
        }
    }

    #[test]
    fn integers_same_signedness_widen() {
        assert_eq!(sup(Int8, Int16), Some(Int16));
        assert_eq!(sup(Int32, Int64), Some(Int64));
        assert_eq!(sup(UInt8, UInt32), Some(UInt32));
    }

    #[test]
    fn integers_mixed_signedness() {
        // Same width → one step wider signed.
        assert_eq!(sup(Int8, UInt8), Some(Int16));
        assert_eq!(sup(Int16, UInt16), Some(Int32));
        assert_eq!(sup(Int32, UInt32), Some(Int64));

        // Signed wider than unsigned → signed already covers it.
        assert_eq!(sup(Int16, UInt8), Some(Int16));
        assert_eq!(sup(Int32, UInt16), Some(Int32));
        assert_eq!(sup(Int64, UInt32), Some(Int64));

        // Signed narrower than unsigned → widen one step past unsigned.
        assert_eq!(sup(Int8, UInt16), Some(Int32));
        assert_eq!(sup(Int16, UInt32), Some(Int64));

        // Int64 ∪ UInt64 is unsafe.
        assert_eq!(sup(Int64, UInt64), None);
    }

    #[test]
    fn float_promotion() {
        assert_eq!(sup(Float32, Float32), Some(Float32));
        assert_eq!(sup(Float32, Float64), Some(Float64));

        // Small ints survive in f32 (24-bit mantissa).
        assert_eq!(sup(Int8, Float32), Some(Float32));
        assert_eq!(sup(Int16, Float32), Some(Float32));
        assert_eq!(sup(UInt16, Float32), Some(Float32));

        // 32/64-bit ints need f64.
        assert_eq!(sup(Int32, Float32), Some(Float64));
        assert_eq!(sup(UInt32, Float32), Some(Float64));
        assert_eq!(sup(Int64, Float32), Some(Float64));
        assert_eq!(sup(Int64, Float64), Some(Float64));
    }

    #[test]
    fn string_and_binary_families() {
        assert_eq!(sup(Utf8, LargeUtf8), Some(LargeUtf8));
        assert_eq!(sup(Binary, LargeBinary), Some(LargeBinary));
        assert_eq!(sup(Binary, FixedSizeBinary(8)), Some(Binary));
        assert_eq!(sup(LargeBinary, FixedSizeBinary(8)), Some(LargeBinary));
        assert_eq!(sup(FixedSizeBinary(8), FixedSizeBinary(16)), None);

        // Cross-family unions are not allowed.
        assert_eq!(sup(Utf8, Binary), None);
        assert_eq!(sup(Utf8, Int32), None);
        assert_eq!(sup(Boolean, Int32), None);
    }

    #[test]
    fn inspection_flags() {
        assert!(DataType::int32().is_integer());
        assert!(DataType::int32().is_signed_integer());
        assert!(!DataType::int32().is_unsigned_integer());
        assert!(DataType::uint32().is_unsigned_integer());
        assert!(DataType::float64().is_floating());
        assert!(DataType::float64().is_numeric());
        assert!(!DataType::boolean().is_numeric());
        assert!(DataType::utf8().is_string());
        assert!(DataType::binary().is_binary());
        assert!(DataType::fixed_size_binary(4).is_binary());

        // Float16 is intentionally *not* supported; make sure the inspection
        // flags agree with `is_supported_primitive`.
        let f16 = DataType::from_arrow(Float16);
        assert!(!f16.is_floating());
        assert!(!f16.is_supported_primitive());
    }

    #[test]
    fn field_derives() {
        let a = Field::new("x", DataType::int32(), true);
        let b = Field::new("x", DataType::int32(), true);
        let c = Field::new("x", DataType::int32(), false);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}