use safety_net::{
    attribute::Parameter,
    circuit::{Identifier, Instantiable, Net},
    logic::Logic,
};
use std::str::FromStr;

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
        if (id.get_name() != "FDRE")
            && (id.get_name() != "FDSE")
            && (id.get_name() != "FDPE")
            && (id.get_name() != "FDCE")
        {
            let name: &str = id.get_name();
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
            &_ => unreachable!(),
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
            Some(Parameter::Logic(self.init_value))
        } else {
            None
        }
    }

    fn set_parameter(&mut self, id: &Identifier, val: Parameter) -> Option<Parameter> {
        if !self.has_parameter(id) {
            return None;
        }

        let old = Some(Parameter::Logic(self.init_value));

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
            Parameter::Logic(self.init_value),
        ))
    }

    fn from_constant(_val: Logic) -> Option<Self> {
        None
    }

    fn get_constant(&self) -> Option<Logic> {
        None
    }

    fn is_seq(&self) -> bool {
        true
    }
}

#[test]
fn flipflop_test() {
    let mut ff = FlipFlop::new("FDRE".into(), Logic::from_str("1'b0").unwrap());
    assert_eq!(ff.get_name(), &"FDRE".into());
    let input_ports: Vec<_> = ff.get_input_ports().into_iter().collect();
    assert_eq!(input_ports[0], &Net::new_logic("C".into()));
    assert_eq!(input_ports[1], &Net::new_logic("CE".into()));
    assert_eq!(input_ports[2], &Net::new_logic("R".into()));
    assert_eq!(input_ports[3], &Net::new_logic("D".into()));
    let output_ports: Vec<_> = ff.get_output_ports().into_iter().collect();
    assert_eq!(output_ports[0], &Net::new_logic("Q".into()));
    let params: Vec<_> = ff.parameters().collect();
    assert_eq!(params[0].0, Identifier::new("INIT".to_string()));
    assert_eq!(
        ff.set_parameter(
            &"INIT".into(),
            Parameter::Logic(Logic::from_str("1'b1").unwrap())
        ),
        Some(Parameter::Logic(Logic::from_str("1'b0").unwrap()))
    );
    assert_eq!(
        ff.get_parameter(&"INIT".into()),
        Some(Parameter::Logic(Logic::from_str("1'b1").unwrap()))
    );
    assert!(ff.is_seq());
}
