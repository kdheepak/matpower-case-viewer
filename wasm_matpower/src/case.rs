// Version 2

// https://codeandbitters.com/lets-build-a-parser/

use std::fmt;

use anyhow::{anyhow, Result};
use escape8259::unescape;
use nom::{
  self,
  branch::alt,
  bytes::complete::{is_not, take_till, take_until, take_until1, take_while1},
  character::complete::{alpha1, alphanumeric1, char, multispace0, one_of},
  combinator::{fail, map, map_res, opt, recognize, value},
  error::{ContextError, ErrorKind, ParseError, VerboseError},
  multi::{fold_many1, many0, many1, separated_list0, separated_list1},
  sequence::{delimited, pair, preceded, terminated, tuple},
  Err, Finish, IResult, Parser,
};
use nom_locate::{position, LocatedSpan};
use nom_supreme::{
  error::{BaseErrorKind, ErrorTree, StackContext},
  final_parser::ExtractContext,
  tag::complete::tag,
  ParserExt,
};
use serde::{Deserialize, Serialize};

type Span<'a> = LocatedSpan<&'a str>;
type PError<'a> = ErrorTree<Span<'a>>;
type PResult<'a, O> = nom::IResult<Span<'a>, O, PError<'a>>;
#[derive(Debug)]
pub struct Error<'a>(nom::Err<PError<'a>>);

impl<'a> fmt::Display for Error<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match &self.0 {
      nom::Err::Error(e) => write!(f, "{}", e),
      nom::Err::Failure(e) => write!(f, "{}", e),
      nom::Err::Incomplete(_) => write!(f, "{:?}", self.0),
    }
  }
}

fn report_error(v: Vec<(Span, StackContext)>, fragment: Span, kind: BaseErrorKind) -> String {
  let mut s = String::new();
  for (i, c) in v.into_iter().rev() {
    let line = fragment.location_line();
    let whole_line = fragment.lines().nth(0);
    let k = if let BaseErrorKind::Expected(k) = kind { format!(" (Expected {})", k) } else { "".to_string() };
    s = format!(
      "{}Error {}{} on line {line}:\n\n{code}\n{marker_padding}^\n\n",
      s,
      c,
      k,
      line = line,
      code = whole_line.unwrap_or("( unknown input )"),
      marker_padding = " ".repeat(fragment.get_utf8_column() - 1)
    );
  }
  s
}

fn usize(i: Span) -> PResult<usize> {
  let parser = recognize(many1(terminated(one_of("0123456789"), many0(char('_')))));
  map(parser, |s: Span| s.fragment().parse().unwrap()).context("usize").parse(i)
}

#[test]
fn test_usize() {
  assert_eq!(usize(Span::from("020")).unwrap().1, 20);
}

fn decimal(i: Span) -> PResult<f64> {
  let parser = recognize(many1(terminated(one_of("0123456789"), many0(char('_')))));
  map(parser, |s: Span| s.fragment().parse::<f64>().unwrap()).context("decimal").parse(i)
}

fn float(i: Span) -> PResult<f64> {
  let parser = alt((
    recognize(tag("Inf")),
    recognize(tag("-Inf")),
    recognize(tuple((opt(one_of("+-")), char('.'), decimal, opt(tuple((one_of("eE"), opt(one_of("+-")), decimal)))))),
    recognize(tuple((
      opt(one_of("+-")),
      decimal,
      opt(preceded(char('.'), decimal)),
      one_of("eE"),
      opt(one_of("+-")),
      decimal,
    ))),
    recognize(tuple((opt(one_of("+-")), decimal, char('.'), opt(decimal)))),
    recognize(tuple((opt(one_of("+-")), decimal))),
  ));
  map(parser, |s| s.fragment().parse::<f64>().unwrap()).context("float").parse(i)
}

#[test]
fn test_float() {
  assert_eq!(float("-360.0".into()).unwrap().1, -360.0);
  assert_eq!(float("-360".into()).unwrap().1, -360.0);
  assert_eq!(float(".02".into()).unwrap().1, 0.02);
  assert_eq!(float("1.0".into()).unwrap().1, 1.0);
  assert_eq!(float("1".into()).unwrap().1, 1.0);
  assert_eq!(float("42.32e32".into()).unwrap().1, 42.32e32);
  assert_eq!(float("Inf".into()).unwrap().1, f64::INFINITY);
  assert_eq!(float("-Inf".into()).unwrap().1, f64::NEG_INFINITY);
}

// String

fn is_nonescaped_string_char(c: char) -> bool {
  let cv = c as u32;
  (cv >= 0x20) && (cv != 0x27) && (cv != 0x5C)
}

// One or more unescaped text characters
fn nonescaped_string(i: Span) -> PResult<Span> {
  take_while1(is_nonescaped_string_char).context("nonescaped_string").parse(i)
}

fn escape_code(i: Span) -> PResult<Span> {
  recognize(pair(
    tag("\\"),
    alt((tag("'"), tag("\\"), tag("/"), tag("b"), tag("f"), tag("n"), tag("r"), tag("t"), tag("u"))),
  ))
  .context("escape_code")
  .parse(i)
}

fn string_body(i: Span) -> PResult<Span> {
  recognize(many1(alt((nonescaped_string, escape_code)))).context("string_body").parse(i)
}

fn string(i: Span) -> PResult<Span> {
  delimited(tag("'"), string_body.context("String"), tag("'")).context("string").parse(i)
}

#[test]
fn test_string() {
  assert_eq!(string("'WHuntngd V2'".into()).unwrap().1.fragment(), &"WHuntngd V2");
  assert!(string("".into()).is_err());
  assert!(string("abs".into()).is_err());
}

fn ws<F, I, O, E>(f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
  F: FnMut(I) -> IResult<I, O, E>,
  I: nom::InputTakeAtPosition,
  <I as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
  E: nom::error::ParseError<I>,
{
  delimited(multispace0, f, multispace0)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
enum BusType {
  PQ = 1,
  PV = 2,
  Ref = 3,
  Isolated = 4,
}

fn bus_type(i: Span) -> PResult<BusType> {
  alt((
    value(BusType::PQ, tag("1")),
    value(BusType::PV, tag("2")),
    value(BusType::Ref, tag("3")),
    value(BusType::Isolated, tag("4")),
  ))
  .context("bus_type")
  .parse(i)
}

#[test]
fn test_bus_type() {
  assert_eq!(bus_type("1".into()).unwrap().1, BusType::PQ);
  assert_eq!(bus_type("2".into()).unwrap().1, BusType::PV);
  assert_eq!(bus_type("3".into()).unwrap().1, BusType::Ref);
  assert_eq!(bus_type("4".into()).unwrap().1, BusType::Isolated);
  assert!(bus_type("5".into()).is_err());
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
struct Bus {
  idx: usize,             // bus number
  bus_type: BusType,      // BusType
  pd: f64,                // real power demand (MW)
  qd: f64,                // reactive power demand (MVAr)
  shunt_conductance: f64, // MW demanded at V = 1.0 p.u.
  shunt_susceptance: f64, // MVar injected at V = 1.0 p.u.
  area: usize,            // area
  voltage_mag: f64,       // p.u.
  voltage_ang: f64,       // degrees
  base_kv: f64,           // kV
  zone: usize,            // loss zone
  v_max: f64,             // p.u.
  v_min: f64,             // p.u.
  lam_p: Option<f64>,     // Lagrange multiplier u/MW
  lam_q: Option<f64>,     // Lagrange multiplier u/MVAr
  mu_vmax: Option<f64>,   // Kuhn Tucker multiplier u/p.u.
  mu_vmin: Option<f64>,   // Kuhn Tucker multiplier u/p.u.
}

fn bus(i: Span) -> PResult<Bus> {
  let parser = terminated(
    tuple((
      ws(usize),      // bus_i
      ws(bus_type),   // type
      ws(float),      // Pd
      ws(float),      // Qd
      ws(float),      // Gs
      ws(float),      // Bs
      ws(usize),      // area
      ws(float),      // Vm
      ws(float),      // Va
      ws(float),      // baseKV
      ws(usize),      // zone
      ws(float),      // Vmax
      ws(float),      // Vmin
      opt(ws(float)), // lam_p
      opt(ws(float)), // lam_q
      opt(ws(float)), // mu_vmax
      opt(ws(float)), // mu_vmin
    )),
    tuple((ws(tag(";")), opt(many0(ws(comment))))),
  );
  map(parser, |bus| {
    Bus {
      idx: bus.0,
      bus_type: bus.1,
      pd: bus.2,
      qd: bus.3,
      shunt_conductance: bus.4,
      shunt_susceptance: bus.5,
      area: bus.6,
      voltage_mag: bus.7,
      voltage_ang: bus.8,
      base_kv: bus.9,
      zone: bus.10,
      v_max: bus.11,
      v_min: bus.12,
      lam_p: bus.13,
      lam_q: bus.14,
      mu_vmax: bus.15,
      mu_vmin: bus.16,
    }
  })
  .context("bus")
  .parse(i)
}

#[test]
fn test_bus() {
  assert_eq!(bus("	1	2	51	27	0	0	1	0.955	10.67	138	1	1.06	0.94;".into()).unwrap().1, Bus {
    idx: 1,
    bus_type: BusType::PV,
    pd: 51.0,
    qd: 27.0,
    shunt_conductance: 0.0,
    shunt_susceptance: 0.0,
    area: 1,
    voltage_mag: 0.955,
    voltage_ang: 10.67,
    base_kv: 138.0,
    zone: 1,
    v_max: 1.06,
    v_min: 0.94,
    lam_p: None,
    lam_q: None,
    mu_vmax: None,
    mu_vmin: None,
  });
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
struct Gen {
  gen: usize,           // bus 1 bus number
  pg: f64,              // 2 real power output (mw)
  qg: f64,              // 3 reactive power output (mvar)
  qmax: f64,            // 4 maximum reactive power output (mvar)
  qmin: f64,            // 5 minimum reactive power output (mvar)
  vg: f64,              // 6 voltage magnitude setpoint (p.u.)
  mbase: f64,           // 7 total mva base of machine, defaults to basemva
  gen_status: usize,    // status 8 machine status, > 0 = machine in-service ≤0 = machine out-of-service
  pmax: f64,            // 9 maximum real power output (mw)
  pmin: f64,            // 10 minimum real power output (mw)
  pc1: f64,             // 11 lower real power output of pq capability curve (mw)
  pc2: f64,             // 12 upper real power output of pq capability curve (mw)
  qc1min: f64,          // 13 minimum reactive power output at pc1 (mvar)
  qc1max: f64,          // 14 maximum reactive power output at pc1 (mvar)
  qc2min: f64,          // 15 minimum reactive power output at pc2 (mvar)
  qc2max: f64,          // 16 maximum reactive power output at pc2 (mvar)
  ramp_agc: f64,        // 17 ramp rate for load following/agc (mw/min)
  ramp_10: f64,         // 18 ramp rate for 10 minute reserves (mw)
  ramp_30: f64,         // 19 ramp rate for 30 minute reserves (mw)
  ramp_q: f64,          // 20 ramp rate for reactive power (2 sec timescale) (mvar/min)
  apf: f64,             // 21 area participation factor
  mu_pmax: Option<f64>, // 22 kuhn-tucker multiplier on upper pg limit (u/mw)
  mu_pmin: Option<f64>, // 23 kuhn-tucker multiplier on lower pg limit (u/mw)
  mu_qmax: Option<f64>, // 24 kuhn-tucker multiplier on upper qg limit (u/mvar)
  mu_qmin: Option<f64>, // 25 kuhn-tucker multiplier on lower qg limit (u/mvar)
}

fn gen(i: Span) -> PResult<Gen> {
  let parser = terminated(
    tuple((
      tuple((
        ws(usize), // gen
        ws(float), // pg
        ws(float), // qg
        ws(float), // qmax
        ws(float), // qmin
        ws(float), // vg
        ws(float), // mbase
        ws(usize), // gen_status
        ws(float), // pmax
        ws(float), // pmin
        ws(float), // pc1
        ws(float), // pc2
        ws(float), // qc1min
        ws(float), // qc1max
        ws(float), // qc2min
        ws(float), // qc2max
        ws(float), // ramp_agc
        ws(float), // ramp_10
        ws(float), // ramp_30
        ws(float), // ramp_q
        ws(float), // apf
      )),
      tuple((
        opt(ws(float)), // mu_pmax
        opt(ws(float)), // mu_pmin
        opt(ws(float)), // mu_qmax
        opt(ws(float)), // mu_qmin
      )),
    )),
    tuple((ws(tag(";")), opt(many0(ws(comment))))),
  );
  map(parser, |gen| {
    Gen {
      gen: gen.0 .0,
      pg: gen.0 .1,
      qg: gen.0 .2,
      qmax: gen.0 .3,
      qmin: gen.0 .4,
      vg: gen.0 .5,
      mbase: gen.0 .6,
      gen_status: gen.0 .7,
      pmax: gen.0 .8,
      pmin: gen.0 .9,
      pc1: gen.0 .10,
      pc2: gen.0 .11,
      qc1min: gen.0 .12,
      qc1max: gen.0 .13,
      qc2min: gen.0 .14,
      qc2max: gen.0 .15,
      ramp_agc: gen.0 .16,
      ramp_10: gen.0 .17,
      ramp_30: gen.0 .18,
      ramp_q: gen.0 .19,
      apf: gen.0 .20,
      mu_pmax: gen.1 .0,
      mu_pmin: gen.1 .1,
      mu_qmax: gen.1 .2,
      mu_qmin: gen.1 .3,
    }
  })
  .context("gen")
  .parse(i)
}

#[test]
fn test_gen() {
  assert_eq!(gen("	1	0	0	15	-5	0.955	100	1	100	0	0	0	0	0	0	0	0	0	0	0	0;".into()).unwrap().1, Gen {
    gen: 1,
    pg: 0.0,
    qg: 0.0,
    qmax: 15.0,
    qmin: -5.0,
    vg: 0.955,
    mbase: 100.0,
    gen_status: 1,
    pmax: 100.0,
    pmin: 0.0,
    pc1: 0.0,
    pc2: 0.0,
    qc1min: 0.0,
    qc1max: 0.0,
    qc2min: 0.0,
    qc2max: 0.0,
    ramp_agc: 0.0,
    ramp_10: 0.0,
    ramp_30: 0.0,
    ramp_q: 0.0,
    apf: 0.0,
    mu_pmax: None,
    mu_pmin: None,
    mu_qmax: None,
    mu_qmin: None,
  });
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
struct Branch {
  f_bus: f64,             // 1 “from” bus number
  t_bus: f64,             // 2 “to” bus number
  br_r: f64,              // 3 resistance (p.u.)
  br_x: f64,              // 4 reactance (p.u.)
  br_b: f64,              // 5 total line charging susceptance (p.u.)
  rate_a: f64,            // 6 mva rating a (long term rating), set to 0 for unlimited
  rate_b: f64,            // 7 mva rating b (short term rating), set to 0 for unlimited
  rate_c: f64,            // 8 mva rating c (emergency rating), set to 0 for unlimited
  tap: f64,               // 9 transformer off nominal turns ratio
  shift: f64,             // 10 transformer phase shift angle (degrees), positive ⇒ delay
  br_status: f64,         // 11 initial branch status, 1 = in-service, 0 = out-of-service
  angmin: f64,            // 12 minimum angle difference, θf −θt (degrees)
  angmax: f64,            // 13 maximum angle difference, θf −θt (degrees)
  pf: Option<f64>,        // 14 real power injected at “from” bus end (mw)
  qf: Option<f64>,        // 15 reactive power injected at “from” bus end (mvar)
  pt: Option<f64>,        // 16 real power injected at “to” bus end (mw)
  qt: Option<f64>,        // 17 reactive power injected at “to” bus end (mvar)
  mu_sf: Option<f64>,     // 18 kuhn-tucker multiplier on mva limit at “from” bus (u/mva)
  mu_st: Option<f64>,     // 19 kuhn-tucker multiplier on mva limit at “to” bus (u/mva)
  mu_angmin: Option<f64>, // 20 kuhn-tucker multiplier lower angle difference limit (u/degree)
  mu_angmax: Option<f64>, // 21 kuhn-tucker multiplier upper angle difference limit (u/degree)
}

fn branch(i: Span) -> PResult<Branch> {
  let parser = terminated(
    tuple((
      ws(float),      // f_bus
      ws(float),      // t_bus
      ws(float),      // br_r
      ws(float),      // br_x
      ws(float),      // br_b
      ws(float),      // rate_a
      ws(float),      // rate_b
      ws(float),      // rate_c
      ws(float),      // tap
      ws(float),      // shift
      ws(float),      // br_status
      ws(float),      // angmin
      ws(float),      // angmax
      opt(ws(float)), // pf
      opt(ws(float)), // qf
      opt(ws(float)), // pt
      opt(ws(float)), // qt
      opt(ws(float)), // mu_sf
      opt(ws(float)), // mu_st
      opt(ws(float)), // mu_angmin
      opt(ws(float)), // mu_angmax
    )),
    tuple((ws(tag(";")), opt(many0(ws(comment))))),
  );
  map(parser, |branch| {
    Branch {
      f_bus: branch.0,
      t_bus: branch.1,
      br_r: branch.2,
      br_x: branch.3,
      br_b: branch.4,
      rate_a: branch.5,
      rate_b: branch.6,
      rate_c: branch.7,
      tap: branch.8,
      shift: branch.9,
      br_status: branch.10,
      angmin: branch.11,
      angmax: branch.12,
      pf: branch.13,
      qf: branch.14,
      pt: branch.15,
      qt: branch.16,
      mu_sf: branch.17,
      mu_st: branch.18,
      mu_angmin: branch.19,
      mu_angmax: branch.20,
    }
  })
  .context("branch")
  .parse(i)
}

#[test]
fn test_branch() {
  assert_eq!(branch("	1	2	0.0303	0.0999	0.0254	0	0	0	0	0	1	-360	360;".into()).unwrap().1, Branch {
    f_bus: 1.0,
    t_bus: 2.0,
    br_r: 0.0303,
    br_x: 0.0999,
    br_b: 0.0254,
    rate_a: 0.0,
    rate_b: 0.0,
    rate_c: 0.0,
    tap: 0.0,
    shift: 0.0,
    br_status: 1.0,
    angmin: -360.0,
    angmax: 360.0,
    pf: None,
    qf: None,
    pt: None,
    qt: None,
    mu_sf: None,
    mu_st: None,
    mu_angmin: None,
    mu_angmax: None,
  });
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
enum CostModel {
  PiecewiseLinear = 1,
  Polynomial = 2,
}

fn cost_model(i: Span) -> PResult<CostModel> {
  alt((value(CostModel::PiecewiseLinear, tag("1")), value(CostModel::Polynomial, tag("2"))))
    .context("cost_model")
    .parse(i)
}

#[test]
fn test_cost_model() {
  assert_eq!(cost_model("1".into()).unwrap().1, CostModel::PiecewiseLinear);
  assert_eq!(cost_model("2".into()).unwrap().1, CostModel::Polynomial);
  assert!(cost_model("5".into()).is_err());
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct GenCost {
  model: CostModel,
  startup: f64,
  shutdown: f64,
  ncost: usize, /* number N = n + 1 of data points defining an n-segment piecewise linear cost function,
                 * or of coefficients defining an n-th order polynomial cost function */
  cost: Vec<f64>,
}

fn gen_cost(i: Span) -> PResult<GenCost> {
  let parser = terminated(
    tuple((ws(cost_model), ws(float), ws(float), ws(usize), separated_list1(one_of("\t "), float))),
    tuple((ws(tag(";")), opt(many0(ws(comment))))),
  );
  map(parser, |gencost| {
    let gc = GenCost { model: gencost.0, startup: gencost.1, shutdown: gencost.2, ncost: gencost.3, cost: gencost.4 };
    match gc.model {
      CostModel::Polynomial => assert!(gc.ncost == gc.cost.len()),
      CostModel::PiecewiseLinear => assert!(gc.ncost * 2 == gc.cost.len()),
    }
    gc
  })
  .context("gen_cost")
  .parse(i)
}

#[test]
fn test_gen_cost() {
  assert_eq!(gen_cost("	2	0	0	3	0.025	16.242	880.2;".into()).unwrap().1, GenCost {
    model: CostModel::Polynomial,
    startup: 0.0,
    shutdown: 0.0,
    ncost: 3,
    cost: vec![0.025, 16.242, 880.2,],
  });
  assert_eq!(gen_cost("	1	0	0	2	0.025	0.025	16.242	880.2;".into()).unwrap().1, GenCost {
    model: CostModel::PiecewiseLinear,
    startup: 0.0,
    shutdown: 0.0,
    ncost: 2,
    cost: vec![0.025, 0.025, 16.242, 880.2,],
  });
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
enum ServiceStatus {
  OutOfService = 0,
  InService = 1,
}

fn service_status(i: Span) -> PResult<ServiceStatus> {
  alt((value(ServiceStatus::OutOfService, tag("0")), value(ServiceStatus::InService, tag("1"))))
    .context("service_status")
    .parse(i)
}

#[test]
fn test_service_status() {
  assert_eq!(service_status("0".into()).unwrap().1, ServiceStatus::OutOfService);
  assert_eq!(service_status("1".into()).unwrap().1, ServiceStatus::InService);
  assert!(service_status("5".into()).is_err());
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
struct DcLine {
  f_bus: usize,             // 1 “from” bus number
  t_bus: usize,             // 2 “to” bus number
  br_status: ServiceStatus, // 3 initial branch status, 1 = in-service, 0 = out-of-service
  pf: f64,                  // †4 real power flow at “from” bus end (mw), “from” → “to”
  pt: f64,                  // †5 real power flow at “to” bus end (mw), “from” → “to”
  qf: f64,                  // †6 reactive power injected into “from” bus (mvar)
  qt: f64,                  // †7 reactive power injected into “to” bus (mvar)
  vf: f64,                  // 8 voltage magnitude setpoint at “from” bus (p.u.)
  vt: f64,                  // 9 voltage magnitude setpoint at “to” bus (p.u.)
  pmin: f64,                // 10 if positive (negative), lower limit on pf (pt)
  pmax: f64,                // 11 if positive (negative), upper limit on pf (pt)
  qminf: f64,               // 12 lower limit on reactive power injection into “from” bus (mvar)
  qmaxf: f64,               // 13 upper limit on reactive power injection into “from” bus (mvar)
  qmint: f64,               // 14 lower limit on reactive power injection into “to” bus (mvar)
  qmaxt: f64,               // 15 upper limit on reactive power injection into “to” bus (mvar)
  loss0: f64,               // 16 coefficient l0 of constant term of linear loss function (mw)
  loss1: f64,               // 17 coefficient l1 of linear term of linear loss function (mw/mw)
  mu_pmin: Option<f64>,     // ‡18 kuhn-tucker multiplier on lower flow limit at “from” bus (u/mw)
  mu_pmax: Option<f64>,     // ‡19 kuhn-tucker multiplier on upper flow limit at “from” bus (u/mw)
  mu_qminf: Option<f64>,    // ‡20 kuhn-tucker multiplier on lower var limit at “from” bus (u/mvar)
  mu_qmaxf: Option<f64>,    // ‡21 kuhn-tucker multiplier on upper var limit at “from” bus (u/mvar)
  mu_qmint: Option<f64>,    // ‡22 kuhn-tucker multiplier on lower var limit at “to” bus (u/mvar)
  mu_qmaxt: Option<f64>,    // ‡23 kuhn-tucker multiplier on upper var limit at “to” bus (u/mvar)
}

fn dcline(i: Span) -> PResult<DcLine> {
  let parser = terminated(
    tuple((
      tuple((
        ws(usize),          // f_bus
        ws(usize),          // t_bus
        ws(service_status), // br_status
        ws(float),          // pf
        ws(float),          // pt
        ws(float),          // qf
        ws(float),          // qt
        ws(float),          // vf
        ws(float),          // vt
        ws(float),          // pmin
        ws(float),          // pmax
        ws(float),          // qminf
        ws(float),          // qmaxf
        ws(float),          // qmint
        ws(float),          // qmaxt
        ws(float),          // loss0
        ws(float),          // loss1
      )),
      tuple((
        opt(ws(float)), // mu_pmin
        opt(ws(float)), // mu_pmax
        opt(ws(float)), // mu_qminf
        opt(ws(float)), // mu_qmaxf
        opt(ws(float)), // mu_qmint
        opt(ws(float)), // mu_qmaxt
      )),
    )),
    tuple((ws(tag(";")), opt(many0(ws(comment))))),
  );
  map(parser, |dcline| {
    DcLine {
      f_bus: dcline.0 .0,
      t_bus: dcline.0 .1,
      br_status: dcline.0 .2,
      pf: dcline.0 .3,
      pt: dcline.0 .4,
      qf: dcline.0 .5,
      qt: dcline.0 .6,
      vf: dcline.0 .7,
      vt: dcline.0 .8,
      pmin: dcline.0 .9,
      pmax: dcline.0 .10,
      qminf: dcline.0 .11,
      qmaxf: dcline.0 .12,
      qmint: dcline.0 .13,
      qmaxt: dcline.0 .14,
      loss0: dcline.0 .15,
      loss1: dcline.0 .16,
      mu_pmin: dcline.1 .0,
      mu_pmax: dcline.1 .1,
      mu_qminf: dcline.1 .2,
      mu_qmaxf: dcline.1 .3,
      mu_qmint: dcline.1 .4,
      mu_qmaxt: dcline.1 .5,
    }
  })
  .context("dcline")
  .parse(i)
}

#[test]
fn test_dcline() {
  assert_eq!(dcline("	2060653	66353	1	500	465.35	-370.36	-369.49	0.96984	0.92617	500	500	-370.36	-370.36	-369.49	-369.49	0	0	0.0000	0.0000	0.0000	0.0000	0.0000	0.0000;".into()).unwrap().1,
        DcLine {
            f_bus: 2060653,
            t_bus: 66353,
            br_status: ServiceStatus::InService,
            pf: 500.0,
            pt: 465.35,
            qf: -370.36,
            qt: -369.49,
            vf: 0.96984,
            vt: 0.92617,
            pmin: 500.0,
            pmax: 500.0,
            qminf: -370.36,
            qmaxf: -370.36,
            qmint: -369.49,
            qmaxt: -369.49,
            loss0: 0.0,
            loss1: 0.0,
            mu_pmin: Some(
                0.0,
            ),
            mu_pmax: Some(
                0.0,
            ),
            mu_qminf: Some(
                0.0,
            ),
            mu_qmaxf: Some(
                0.0,
            ),
            mu_qmint: Some(
                0.0,
            ),
            mu_qmaxt: Some(
                0.0,
            ),
        });
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
enum Version {
  Version1 = 1,
  Version2 = 2,
}

fn version(i: Span) -> PResult<Version> {
  alt((
    value(Version::Version1, ws(tag("1"))),
    value(Version::Version2, ws(tag("2"))),
    value(Version::Version1, ws(tag("'1'"))),
    value(Version::Version2, ws(tag("'2'"))),
  ))
  .context("version")
  .parse(i)
}

#[test]
fn test_version() {
  assert_eq!(version("1".into()).unwrap().1, Version::Version1);
  assert_eq!(version("2".into()).unwrap().1, Version::Version2);
  assert_eq!(version("'1'".into()).unwrap().1, Version::Version1);
  assert_eq!(version("'2'".into()).unwrap().1, Version::Version2);
  assert!(version("5".into()).is_err());
}

fn comment(i: Span) -> PResult<()> {
  value(
    (), // Output is thrown away.
    pair(char('%'), is_not("\n\r")),
  )
  .context("comment")
  .parse(i)
}

pub fn identifier(i: Span) -> PResult<Span> {
  recognize(pair(alt((alpha1, tag("_"))), many0(alt((alphanumeric1, tag("_")))))).context("identifier").parse(i)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Case {
  name: String,
  version: Version,
  base_mva: f64,
  bus: Vec<Bus>,
  gen: Vec<Gen>,
  gencost: Vec<GenCost>,
  branch: Vec<Branch>,
  dcline: Vec<DcLine>,
  bus_name: Vec<String>,
}

fn get_name(i: Span) -> PResult<String> {
  let (i, _) = if i.starts_with("function") {
    ws(tag("function")).context("function").parse(i)?
  } else {
    take_until1("function").context("function").parse(i)?
  };
  let (i, _) = take_until1("=")(i)?;
  map(preceded(ws(tag("=")), identifier), |s| s.to_string()).context("function").parse(i)
}

#[test]
fn test_get_name() {
  assert_eq!(get_name("function mpc    = case118".into()).unwrap().1, "case118");
  assert_eq!(get_name("function chgtab = scenarios_ACTIVSg2000".into()).unwrap().1, "scenarios_ACTIVSg2000");
}

fn get_version(i: Span) -> PResult<Version> {
  let (i, _) = take_until1("mpc.version").context("get_version").parse(i)?;
  let (i, _) = tag("mpc.version").context("get_version").parse(i)?;
  preceded(ws(tag("=")), ws(version)).context("get_version").parse(i)
}

fn get_base_mva(i: Span) -> PResult<f64> {
  let (i, _) = take_until1("mpc.baseMVA").context("get_base_mva").parse(i)?;
  let (i, _) = tag("mpc.baseMVA").context("get_base_mva").parse(i)?;
  preceded(ws(tag("=")), ws(float)).context("get_base_mva").parse(i)
}

fn get_bus(i: Span) -> PResult<Vec<Bus>> {
  let (i, _) = take_until1("mpc.bus").context("get_bus").parse(i)?;
  let (i, _) = tag("mpc.bus").context("get_bus").parse(i)?;
  preceded(
    ws(tag("=")),
    delimited(
      tuple((ws(tag("[")), opt(ws(comment)))),
      fold_many1(bus, Vec::new, |mut acc: Vec<_>, item| {
        acc.push(item);
        acc
      }),
      ws(tag("]")),
    ),
  )
  .context("get_bus")
  .parse(i)
}

fn get_gen(i: Span) -> PResult<Vec<Gen>> {
  let (i, _) = take_until1("mpc.gen").context("get_gen").parse(i)?;
  let (i, _) = tag("mpc.gen").context("get_gen").parse(i)?;
  preceded(
    ws(tag("=")),
    delimited(
      tuple((ws(tag("[")), opt(ws(comment)))),
      fold_many1(gen, Vec::new, |mut acc: Vec<_>, item| {
        acc.push(item);
        acc
      }),
      ws(tag("]")),
    ),
  )
  .context("get_gen")
  .parse(i)
}

fn get_gencost(i: Span) -> PResult<Vec<GenCost>> {
  let (i, _) = take_until("mpc.gencost").context("get_gencost").parse(i)?;
  let (i, _) = tag("mpc.gencost").context("get_gencost").parse(i)?;
  preceded(
    ws(tag("=")),
    delimited(
      tuple((ws(tag("[")), opt(ws(comment)))),
      fold_many1(gen_cost, Vec::new, |mut acc: Vec<_>, item| {
        acc.push(item);
        acc
      }),
      ws(tag("]")),
    ),
  )
  .context("get_gencost")
  .parse(i)
}

fn get_branch(i: Span) -> PResult<Vec<Branch>> {
  let (i, _) = take_until1("mpc.branch").context("get_branch").parse(i)?;
  let (i, _) = tag("mpc.branch").context("get_branch").parse(i)?;
  preceded(
    ws(tag("=")),
    delimited(
      tuple((ws(tag("[")), opt(many0(ws(comment))))),
      fold_many1(branch, Vec::new, |mut acc: Vec<_>, item| {
        acc.push(item);
        acc
      }),
      ws(tag("]")),
    ),
  )
  .context("get_branch")
  .parse(i)
}

fn get_dcline(i: Span) -> PResult<Vec<DcLine>> {
  let (i, _) = take_until("mpc.dcline").context("get_dcline").parse(i)?;
  let (i, _) = tag("mpc.dcline").context("get_dcline").parse(i)?;
  preceded(
    ws(tag("=")),
    delimited(
      tuple((ws(tag("[")), opt(ws(comment)))),
      fold_many1(dcline, Vec::new, |mut acc: Vec<_>, item| {
        acc.push(item);
        acc
      }),
      ws(tag("]")),
    ),
  )
  .context("get_dcline")
  .parse(i)
}

fn get_busname(i: Span) -> PResult<Vec<String>> {
  let (i, _) = take_until1("mpc.bus_name").context("get_busname").parse(i)?;
  let (i, _) = tag("mpc.bus_name").context("get_busname").parse(i)?;
  let (i, v) = preceded(
    ws(tag("=")),
    delimited(
      tuple((ws(tag("{")), opt(ws(comment)))),
      fold_many1(terminated(ws(string), ws(tag(";"))), Vec::new, |mut acc: Vec<_>, item| {
        acc.push(item);
        acc
      }),
      ws(tag("}")),
    ),
  )
  .context("get_busname")
  .parse(i)?;
  Ok((i, v.iter().map(|s| s.fragment().to_owned().to_string()).collect::<Vec<String>>()))
}

#[test]
fn test_get_busname() {
  let data = r#"%% Ignore this line
mpc.bus_name = {
	'Riversde  V2';
	'Pokagon   V2';
  }"#
    .into();

  assert_eq!(get_busname(data).unwrap().1.len(), 2);
}

fn _case(i: &str) -> PResult<Case> {
  let i = Span::from(i);
  let (_, name) = get_name(i)?;
  let (_, version) = get_version(i)?;
  let (_, base_mva) = get_base_mva(i)?;
  let (_, bus) = get_bus(i)?;
  let (_, gen) = get_gen(i)?;
  let (_, branch) = get_branch(i)?;
  let (_, gencost) = get_gencost(i).or_else(|_| Ok(("".into(), vec![])))?;
  let (_, dcline) = get_dcline(i).or_else(|_| Ok(("".into(), vec![])))?;
  let (_, bus_name) = get_busname(i).or_else(|_| Ok(("".into(), vec![])))?;
  Ok(("".into(), Case { name, version, base_mva, bus, gen, gencost, branch, dcline, bus_name }))
}

pub fn case(i: &str) -> Result<Case> {
  let r = _case(i);
  if r.is_err() {
    let mut s = String::new();
    let e = r.unwrap_err();
    if let nom::Err::Error(nom_supreme::error::ErrorTree::Stack { contexts, base }) = e {
      if let nom_supreme::error::ErrorTree::Base { location, kind } = *base {
        s = report_error(contexts, location, kind)
      }
    }
    Err(anyhow!("Unable to build case. {}", s))
  } else {
    Ok(r.unwrap().1)
  }
}

#[test]
fn test_case() {
  let entries = std::fs::read_dir("../../matpower/data/").unwrap().map(|res| res.map(|e| e.path())).collect::<Vec<_>>();
  for f in entries.into_iter() {
    println!("Parsing {:?}", &f.as_ref().unwrap());
    let s = std::fs::read_to_string(f.unwrap()).unwrap();

    let r = _case(&s);
    if r.is_err() {
      dbg!(r.unwrap_err());
    }
  }
}
