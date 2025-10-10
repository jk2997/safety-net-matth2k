use safety_net::{
    netlist::GateNetlist,
    netlist::Gate,
    attribute::Parameter,
    circuit::{Identifier, Instantiable, Net},
    format_id,
    logic::Logic,
    netlist::Netlist,
};
use bitvec::vec::BitVec;
pub use inst_derive::Instantiable;
use std::str::FromStr;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Lut {
    lookup_table: BitVec,
    id: Identifier,
    inputs: Vec<Net>,
    output: Net,
}

impl Lut {
    fn new(k: usize, lookup_table: usize) -> Self {
        let mut bv: BitVec<usize, _> = BitVec::from_element(lookup_table);
        bv.truncate(1 << k);
        Lut {
            lookup_table: bv,
            id: format_id!("LUT{k}"),
            inputs: (0..k).map(|i| Net::new_logic(format_id!("I{i}"))).collect(),
            output: Net::new_logic("O".into()),
        }
    }

    fn invert(&mut self) {
        self.lookup_table = !self.lookup_table.clone();
    }
}

impl Instantiable for Lut {
    fn get_name(&self) -> &Identifier {
        &self.id
    }

    fn get_input_ports(&self) -> impl IntoIterator<Item = &Net> {
        &self.inputs
    }

    fn get_output_ports(&self) -> impl IntoIterator<Item = &Net> {
        std::slice::from_ref(&self.output)
    }

    fn has_parameter(&self, id: &Identifier) -> bool {
        *id == Identifier::new("INIT".to_string())
    }

    fn get_parameter(&self, id: &Identifier) -> Option<Parameter> {
        if self.has_parameter(id) {
            Some(Parameter::BitVec(self.lookup_table.clone()))
        } else {
            None
        }
    }

    fn set_parameter(&mut self, id: &Identifier, val: Parameter) -> Option<Parameter> {
        if !self.has_parameter(id) {
            return None;
        }

        let old = Some(Parameter::BitVec(self.lookup_table.clone()));

        if let Parameter::BitVec(bv) = val {
            self.lookup_table = bv;
        } else {
            panic!("Invalid parameter type for INIT");
        }

        old
    }

    fn parameters(&self) -> impl Iterator<Item = (Identifier, Parameter)> {
        std::iter::once((
            Identifier::new("INIT".to_string()),
            Parameter::BitVec(self.lookup_table.clone()),
        ))
    }

    fn from_constant(val: Logic) -> Option<Self> {
        match val {
            Logic::True => Some(Self {
                lookup_table: BitVec::from_element(1),
                id: "VDD".into(),
                inputs: vec![],
                output: "Y".into(),
            }),
            Logic::False => Some(Self {
                lookup_table: BitVec::from_element(0),
                id: "GND".into(),
                inputs: vec![],
                output: "Y".into(),
            }),
            _ => None,
        }
    }

    fn get_constant(&self) -> Option<Logic> {
        match self.id.to_string().as_str() {
            "VDD" => Some(Logic::True),
            "GND" => Some(Logic::False),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
/// A flip-flop in a digital circuit
struct FlipFlop {
    id: Identifier,
    init_value: Logic,
    q: Net,
    c: Net,
    ce: Net,
    reset: Net,
    d: Net,
}

impl FlipFlop {
    fn new(id: Identifier, init_value: Logic) -> Self {
        if (id.get_name() != "FDRE") &&
           (id.get_name() != "FDSE") &&
           (id.get_name() != "FDPE") &&
           (id.get_name() != "FDCE")
        {
            let name : &str = id.get_name();
            panic!("Unsupported flip-flop type: {name}");
        }
        let q = Net::new_logic("Q".into());
        let c = Net::new_logic("C".into());
        let ce = Net::new_logic("CE".into());
        let reset = Net::new_logic(match id.get_name() {
            "FDRE" => "R".into(),
            "FDSE" => "S".into(),
            "FDPE" => "PRE".into(),
            "FDCE" => "CLR".into(),
            &_ => unreachable!()
        });
        let d = Net::new_logic("D".into());
        FlipFlop {
            id,
            init_value,
            q,
            c,
            ce,
            reset,
            d,
        }
    }
}

impl Instantiable for FlipFlop {
    fn get_name(&self) -> &Identifier {
        &self.id
    }

    fn get_input_ports(&self) -> impl IntoIterator<Item = &Net> {
        vec![&self.c, &self.ce, &self.reset, &self.d]
    }

    fn get_output_ports(&self) -> impl IntoIterator<Item = &Net> {
        std::slice::from_ref(&self.q)
    }

    fn has_parameter(&self, id: &Identifier) -> bool {
        *id == Identifier::new("INIT".to_string())
    }

    fn get_parameter(&self, id: &Identifier) -> Option<Parameter> {
        if self.has_parameter(id) {
            Some(Parameter::Logic(self.init_value.clone()))
        } else {
            None
        }
    }

    fn set_parameter(&mut self, id: &Identifier, val: Parameter) -> Option<Parameter> {
        if !self.has_parameter(id) {
            return None;
        }

        let old = Some(Parameter::Logic(self.init_value.clone()));
  
        if let Parameter::Logic(l) = val {
            self.init_value = l;
        } else {
            panic!("Invalid type for INIT parameter: {val}");
        }

        old
    }

    fn parameters(&self) -> impl Iterator<Item = (Identifier, Parameter)> {
        std::iter::once((
            Identifier::new("INIT".to_string()),
            Parameter::Logic(self.init_value.clone()),
        ))
    }

    fn from_constant(_val: Logic) -> Option<Self> {
        None
    }

    fn get_constant(&self) -> Option<Logic> {
        None
    }
}

#[derive(Debug, Clone, Instantiable)]
enum Cell {
    #[instantiable(constant)]
    Lut(Lut),
    FlipFlop(FlipFlop),
    Gate(Gate),
}


#[test]
fn flipflop_test() {
    let netlist = Netlist::new("test_netlist".to_string());
  
    let clk = netlist.insert_input("clk".into());
    let ce = netlist.insert_input("ce".into());        
    let rst = netlist.insert_input("rst".into());
    let d = netlist.insert_input("d".into());

    let mut flipflop = FlipFlop::new("FDRE".into(), Logic::from_str("1'bx").unwrap());
    flipflop.set_parameter(&"INIT".into(), Parameter::Logic(Logic::from_str("1'b0").unwrap()));
    assert_eq!(flipflop.get_parameter(&"INIT".into()), Some(Parameter::Logic(Logic::from_str("1'b0").unwrap())));
        
    let instance = netlist
        .insert_gate(Cell::FlipFlop(flipflop), 
                    "ff1".into(), &[clk, ce, rst, d])
        .unwrap();
    
    instance.expose_with_name("q".into());
    assert!(netlist.verify().is_ok());
}

fn nand_gate() -> Gate {
    Gate::new_logical("NAND".into(), vec!["A".into(), "B".into()], "Y".into())
}

fn not_gate() -> Gate {
    Gate::new_logical("NOT".into(), vec!["A".into()], "Y".into())
}

#[test]
fn d_flip_flop() {
    let netlist = GateNetlist::new("d_flip_flop".to_string());

    let d = netlist.insert_input("d".into());
    let clk = netlist.insert_input("clk".into());

    let inv_1 = netlist.insert_gate(not_gate(), "not1".into(), &[d.clone()])
        .unwrap();
    let nand_1 = netlist.insert_gate(nand_gate(), "nand_1".into(), &[d, clk.clone()])
        .expect("Failed to insert gate");
    let nand_2 = netlist.insert_gate(nand_gate(), "nand_2".into(), &[inv_1.get_output(0), clk])  
        .expect("Failed to insert gate");
    let nand_3 = netlist.insert_gate_disconnected(nand_gate(), "nand3".into());
    let nand_4 = netlist.insert_gate_disconnected(nand_gate(), "nand4".into());

    nand_3.get_input(0).connect(nand_1.get_output(0));
    nand_3.get_input(1).connect(nand_4.get_output(0));
    nand_4.get_input(0).connect(nand_3.get_output(0));
    nand_4.get_input(1).connect(nand_2.get_output(0));
    nand_3.expose_with_name("q".into());
    assert!(netlist.verify().is_ok());
}