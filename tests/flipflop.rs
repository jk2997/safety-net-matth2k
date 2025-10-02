use safety_net::{
    netlist::GateNetlist,
    netlist::Gate,
    attribute::Parameter,
    circuit::{Identifier, Instantiable, Net},
    logic::Logic,
    netlist::Netlist,
};

#[derive(Debug, Clone)]
/// A flip-flop in a digital circuit
struct FlipFlop {
    id: Identifier,
    init_value: Parameter,
    q: Net,
    c: Net,
    ce: Net,
    reset: Net,
    d: Net,
}

impl FlipFlop {
    fn new(id: Identifier, init_value: u64) -> Self {
        if (id.get_name() != "FDRE") &&
           (id.get_name() != "FDSE") &&
           (id.get_name() != "FDPE") &&
           (id.get_name() != "FDCE")
        {
            let name : &str = id.get_name();
            panic!("Unsupported flip-flop type: {name}");
        }
        if (init_value != 0) && (init_value != 1) {
            panic!("Invalid initial value for flip-flop: {init_value}");
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
            init_value: Parameter::Integer(init_value),
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
            Some(self.init_value.clone())
        } else {
            None
        }
    }

    fn set_parameter(&mut self, id: &Identifier, val: Parameter) -> Option<Parameter> {
        if !self.has_parameter(id) {
            return None;
        }

        let old = self.get_parameter(id);
  
        if let Parameter::Integer(i) = val {
            if i > 1 {
                panic!("Invalid value for INIT parameter: {val}");
            } else {
                self.init_value = Parameter::Integer(i);
            }
        } else {
            panic!("Invalid type for INIT parameter: {val}");
        }

        old
    }

    fn parameters(&self) -> impl Iterator<Item = (Identifier, Parameter)> {
        std::iter::once((
            Identifier::new("INIT".to_string()),
            self.init_value.clone(),
        ))
    }

    fn from_constant(val: Logic) -> Option<Self> {
        None
    }

    fn get_constant(&self) -> Option<Logic> {
        None
    }
}   

#[test]
fn flipflop_test() {
    let netlist = Netlist::new("test_netlist".to_string());
  
    let clk = netlist.insert_input("clk".into());
    let ce = netlist.insert_input("ce".into());        
    let rst = netlist.insert_input("rst".into());
    let d = netlist.insert_input("d".into());
        
    let instance = netlist
        .insert_gate(FlipFlop::new("FDRE".into(), 0), 
                    "ff1".into(), &[clk, ce, rst, d])
        .unwrap();

    instance.expose_with_name("q".into());
}

fn nand_gate() -> Gate {
    Gate::new_logical("NAND".into(), vec!["A".into(), "B".into()], "Y".into())
}

fn not_gate() -> Gate {
    Gate::new_logical("NOT".into(), vec!["A".into()], "Y".into())
}

fn buf_gate() -> Gate {
    Gate::new_logical("BUF".into(), vec!["A".into()], "Y".into())
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