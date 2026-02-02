/*!

  Attributes and parameters for nets and node (gates) in the netlist.

*/

use bitvec::{bitvec, field::BitField, order::Lsb0, vec::BitVec};
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use crate::{
    circuit::Instantiable,
    error::Error,
    logic::Logic,
    netlist::{NetRef, Netlist},
};

/// A Verilog attribute assigned to a net or gate in the netlist: (* dont_touch *)
pub type AttributeKey = String;
/// A Verilog attribute can be assigned a string value: bitvec = (* dont_touch = true *)
pub type AttributeValue = Option<String>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An attribute can add information to instances and wires in string form, like 'dont_touch'
pub struct Attribute {
    k: AttributeKey,
    v: AttributeValue,
}

impl Attribute {
    /// Create a new attribute pair
    pub fn new(k: AttributeKey, v: AttributeValue) -> Self {
        Self { k, v }
    }

    /// Get the key of the attribute
    pub fn key(&self) -> &AttributeKey {
        &self.k
    }

    /// Get the value of the attribute
    pub fn value(&self) -> &AttributeValue {
        &self.v
    }

    /// Map a attribute key-value pairs to the Attribute struct
    pub fn from_pairs(
        iter: impl Iterator<Item = (AttributeKey, AttributeValue)>,
    ) -> impl Iterator<Item = Self> {
        iter.map(|(k, v)| Self::new(k, v))
    }
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = &self.v {
            write!(f, "(* {} = {} *)", self.k, value)
        } else {
            write!(f, "(* {} *)", self.k)
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A dedicated type to parameters for instantiables
pub enum Parameter {
    /// An unsigned integer parameter
    Integer(u64),
    /// A floating-point parameter
    Real(OrderedFloat<f32>),
    /// A bit vector parameter, like for a truth table
    BitVec(BitVec),
    /// A four-state logic parameter
    Logic(Logic),
}

impl FromStr for Parameter {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split("'").collect::<Vec<&str>>();
        if split.len() == 2 {
            let literal = split[1];
            let bitsize = split[0].parse::<u64>().unwrap() as usize;
            if let Some(bitstring) = literal.strip_prefix("b") {
                // Caveat: 1'b1 and 1'b0 are always converted to Parameter::Logic instead of Parameter::BitVec
                if bitstring.len() == 1 {
                    match bitstring {
                        "1" => Ok(Parameter::Logic(Logic::True)),
                        "0" => Ok(Parameter::Logic(Logic::False)),
                        "x" => Ok(Parameter::Logic(Logic::X)),
                        "z" => Ok(Parameter::Logic(Logic::Z)),
                        _ => Err(Error::ParseError(
                            "Invalid bit value: {bitstring}".to_string(),
                        )),
                    }
                } else {
                    Ok(Parameter::bitvec(
                        bitsize,
                        u64::from_str_radix(bitstring, 2).unwrap(),
                    ))
                }
            } else if let Some(bitstring) = literal.strip_prefix("h") {
                Ok(Parameter::bitvec(
                    bitsize,
                    u64::from_str_radix(bitstring, 16)
                        .expect(&format!("bitstring = {}", bitstring)),
                ))
            } else if let Some(bitstring) = literal.strip_prefix("d") {
                Ok(Parameter::bitvec(
                    bitsize,
                    bitstring.parse::<u64>().unwrap(),
                ))
            } else {
                Err(Error::ParseError(
                    "Expected a literal with specific bitwidth/format".to_string(),
                ))
            }
        } else {
            if split.len() == 1 {
                if split[0].contains(&".") {
                    Ok(Parameter::Real(OrderedFloat(
                        split[0].parse::<f32>().unwrap(),
                    )))
                } else {
                    Ok(Parameter::Integer(
                        split[0]
                            .parse::<u64>()
                            .expect(&format!("split = {}", split[0].to_string())),
                    ))
                }
            } else {
                Err(Error::ParseError(
                    "Invalid format for parameter: {s}".to_string(),
                ))
            }
        }
    }
}

impl Eq for Parameter {}

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Parameter::Integer(i) => write!(f, "{i}"),
            Parameter::Real(_r) => todo!(),
            Parameter::BitVec(bv) => {
                if bv.len() >= 4 && bv.len() % 4 == 0 {
                    write!(f, "{}'h", bv.len())?;
                    for n in bv.chunks(4).rev() {
                        let val: u8 = n.load();
                        write!(f, "{:x}", val)?;
                    }
                    Ok(())
                } else {
                    write!(
                        f,
                        "{}'b{}",
                        bv.len(),
                        bv.iter()
                            .rev()
                            .map(|b| if *b { '1' } else { '0' })
                            .collect::<String>()
                    )
                }
            }
            Parameter::Logic(l) => write!(f, "{l}"),
        }
    }
}

impl Parameter {
    /// Create a new integer parameter
    pub fn integer(i: u64) -> Self {
        Self::Integer(i)
    }

    /// Create a new real parameter
    pub fn real(r: f32) -> Self {
        Self::Real(OrderedFloat(r))
    }

    /// Create a new bitvec parameter
    pub fn bitvec(size: usize, val: u64) -> Self {
        if size > 64 {
            panic!("BitVec parameter size cannot be larger than 64");
        }
        let mut bv: BitVec = bitvec!(usize, Lsb0; 0; 64);
        bv[0..64].store::<u64>(val);
        bv.truncate(size);
        Self::BitVec(bv)
    }

    /// Create a new Logic parameter
    pub fn logic(l: Logic) -> Self {
        Self::Logic(l)
    }

    /// Create a new Logic parameter from bool
    pub fn from_bool(b: bool) -> Self {
        Self::Logic(Logic::from_bool(b))
    }
}

/// Filter nodes/nets in the netlist by some attribute, like "dont_touch"
pub struct AttributeFilter<'a, I: Instantiable> {
    // A reference to the underlying netlist
    _netlist: &'a Netlist<I>,
    // The keys to filter by
    keys: Vec<AttributeKey>,
    /// The mapping of netrefs that have this attribute
    map: HashMap<AttributeKey, HashSet<NetRef<I>>>,
    /// Contains a dedup collection of all filtered nodes
    full_set: HashSet<NetRef<I>>,
}

impl<'a, I> AttributeFilter<'a, I>
where
    I: Instantiable,
{
    /// Create a new filter for the netlist
    fn new(netlist: &'a Netlist<I>, keys: Vec<AttributeKey>) -> Self {
        let mut map = HashMap::new();
        let mut full_set = HashSet::new();
        for nr in netlist.objects() {
            for attr in nr.attributes() {
                if keys.contains(attr.key()) {
                    map.entry(attr.key().clone())
                        .or_insert_with(HashSet::new)
                        .insert(nr.clone());
                    full_set.insert(nr.clone());
                }
            }
        }
        Self {
            _netlist: netlist,
            keys,
            map,
            full_set,
        }
    }

    /// Check if an node matches any of the filter keys
    pub fn has(&self, n: &NetRef<I>) -> bool {
        self.map.values().any(|s| s.contains(n))
    }

    /// Return a slice to the keys that were used for filtering
    pub fn keys(&self) -> &[AttributeKey] {
        &self.keys
    }
}

impl<'a, I> IntoIterator for AttributeFilter<'a, I>
where
    I: Instantiable,
{
    type Item = NetRef<I>;

    type IntoIter = std::collections::hash_set::IntoIter<NetRef<I>>;

    fn into_iter(self) -> Self::IntoIter {
        self.full_set.into_iter()
    }
}

/// Returns a filtering of nodes and nets that are marked as 'dont_touch'
pub fn dont_touch_filter<'a, I>(netlist: &'a Netlist<I>) -> AttributeFilter<'a, I>
where
    I: Instantiable,
{
    AttributeFilter::new(netlist, vec!["dont_touch".to_string()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribute_iter() {
        let attributes: [(AttributeKey, AttributeValue); 2] = [
            ("dont_touch".to_string(), Some("true".to_string())),
            ("synthesizable".to_string(), None),
        ];
        let real_attrs: Vec<Attribute> = Attribute::from_pairs(attributes.into_iter()).collect();
        assert_eq!(real_attrs.len(), 2);
        assert_eq!(
            real_attrs.first().unwrap().to_string(),
            "(* dont_touch = true *)"
        );
        assert_eq!(real_attrs.first().unwrap().key(), "dont_touch");
        assert_eq!(
            real_attrs.last().unwrap().to_string(),
            "(* synthesizable *)"
        );
        assert!(real_attrs.last().unwrap().value().is_none());
    }

    #[test]
    fn test_parameter_fmt() {
        let p1 = Parameter::Integer(42);
        // Lsb first
        let p2 = Parameter::BitVec(bitvec![0, 0, 0, 0, 0, 0, 0, 1]);
        let p3 = Parameter::Logic(Logic::from_bool(true));
        let p4 = Parameter::from_bool(true);
        assert_eq!(p1.to_string(), "42");
        assert_eq!(p2.to_string(), "8'h80");
        assert_eq!(p3.to_string(), "1'b1");
        assert_eq!(p4.to_string(), "1'b1");
    }

    #[test]
    fn test_parameter_hex() {
        let p = Parameter::BitVec(bitvec![1, 1, 1, 0, 1, 0, 0, 0]);
        assert_eq!(p.to_string(), "8'h17");
        let p = Parameter::BitVec(bitvec![1, 1, 1, 1, 1, 0, 0, 0]);
        assert_eq!(p.to_string(), "8'h1f");
    }

    #[test]
    fn test_parameter_fromstr_logic() {
        let p = Parameter::from_str("1'b1").unwrap();
        assert_eq!(p, Parameter::Logic(Logic::True));
        let p = Parameter::from_str("1'b0").unwrap();
        assert_eq!(p, Parameter::Logic(Logic::False));
        let p = Parameter::from_str("1'bx").unwrap();
        assert_eq!(p, Parameter::Logic(Logic::X));
        let p = Parameter::from_str("1'bz").unwrap();
        assert_eq!(p, Parameter::Logic(Logic::Z));
        assert!(Parameter::from_str("1'ba").is_err());
    }

    #[test]
    fn test_parameter_fromstr() {
        let p = Parameter::from_str("5'b10101").unwrap();
        assert_eq!(p, Parameter::bitvec(5, 21));
        let p = Parameter::from_str("8'h1A").unwrap();
        assert_eq!(p, Parameter::bitvec(8, 26));
        let p = Parameter::from_str("10'd600").unwrap();
        assert_eq!(p, Parameter::bitvec(10, 600));
        let p = Parameter::from_str("1024.5").unwrap();
        assert_eq!(p, Parameter::Real(OrderedFloat(1024.5)));
        let p = Parameter::from_str("10000").unwrap();
        assert_eq!(p, Parameter::Integer(10000));
        assert!(Parameter::from_str("1'1'1").is_err());
    }
}
