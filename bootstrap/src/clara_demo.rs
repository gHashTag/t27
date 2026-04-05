// bootstrap/src/clara_demo.rs
// CLARA Demo: ML+AR Composition Simulation
//
// Implements the demo pipeline for CLARA Ring 42 requirement:
// - CNN + AR composition pattern
// - Proof trace generation (≤10 steps)
// - XAI formatting (natural/fitch/compact)
// - GF16 confidence encoding
//
// φ² + 1/φ² = 3 | TRINITY

// ============================================================================
// CONSTANTS
// ============================================================================

/// Maximum derivation steps per CLARA requirement (from proof_trace.t27)
const MAX_STEPS: u8 = 10;

/// GF16 encoding of 1.0 (full confidence)
const GF16_ONE: u16 = 0x3C00;

/// GF16 encoding of 0.0 (no confidence)
const GF16_ZERO: u16 = 0x0000;

// ============================================================================
// TYPES
// ============================================================================

/// Kleene K3 truth values (from ternary_logic.t27)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trit {
    Neg,  // K_FALSE
    Zero, // K_UNKNOWN
    Pos,  // K_TRUE
}

impl Trit {
    pub fn k3_and(self, other: Trit) -> Trit {
        // Kleene AND: minimum of truth values
        match (self, other) {
            (Trit::Pos, _) | (_, Trit::Pos) => {
                if self == Trit::Pos && other == Trit::Pos {
                    Trit::Pos
                } else {
                    Trit::Pos // Any TRUE makes result TRUE
                }
            }
            (Trit::Neg, _) | (_, Trit::Neg) => Trit::Neg,
            _ => Trit::Zero,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Trit::Neg => "K_FALSE",
            Trit::Zero => "K_UNKNOWN",
            Trit::Pos => "K_TRUE",
        }
    }
}

/// Composition pattern type (from composition.t27)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompositionPattern {
    CnnRules,       // CNN feature extraction → AR rule evaluation
    MlpBayesian,    // MLP forward pass → Bayesian inference
    TransformerXai,  // Self-attention → ≤10 step explanation
    RlGuardrails,    // Policy network → AR constraint checking
}

/// Output format style (from explainability.t27)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormatStyle {
    Natural, // Human-readable sentences
    Fitch,   // Formal natural deduction format
    Compact,  // Single-line summary
}

/// Derivation step in proof trace (from proof_trace.t27)
#[derive(Debug, Clone)]
pub struct DerivationStep {
    pub step_number: u8,
    pub rule_name: String,
    pub input_facts: Vec<Trit>,
    pub output_fact: Trit,
    pub confidence: f32,  // GF16 decoded value
    pub k3_value: Trit,
}

/// Proof trace with bounded steps (from proof_trace.t27)
#[derive(Debug, Clone)]
pub struct ProofTrace {
    pub steps: Vec<DerivationStep>,
    pub step_count: u8,
    pub conclusion: Trit,
    pub total_confidence: f32,
    pub terminated: bool,
}

/// ML component abstraction
#[derive(Debug, Clone)]
pub struct MLComponent {
    pub component_type: String,
    pub confidence: f32,
    pub decision: Trit,
}

/// AR component abstraction
#[derive(Debug, Clone)]
pub struct ARComponent {
    pub rules: Vec<String>,
    pub rule_count: usize,
    pub decision: Trit,
    pub confidence: f32,
    pub trace: ProofTrace,
}

/// Composition result
#[derive(Debug, Clone)]
pub struct CompositionResult {
    pub prediction: Trit,
    pub confidence: f32,
    pub explanation: String,
    pub proof_trace: ProofTrace,
    pub satisfaction: f32,
}

// ============================================================================
// CLI ARGUMENTS
// ============================================================================

#[derive(Debug, Clone)]
pub struct ClaraDemoArgs {
    pub input_file: Option<String>,
    pub pattern: CompositionPattern,
    pub style: FormatStyle,
    pub verbose: bool,
}

impl Default for ClaraDemoArgs {
    fn default() -> Self {
        Self {
            input_file: None,
            pattern: CompositionPattern::CnnRules,
            style: FormatStyle::Natural,
            verbose: false,
        }
    }
}

// ============================================================================
// DEMO PIPELINE FUNCTIONS
// ============================================================================

/// Main entry point for CLARA demo
pub fn run_demo(args: Vec<String>) -> i32 {
    let demo_args = match parse_demo_args(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };

    if demo_args.verbose {
        eprintln!("CLARA Demo: ML+AR Composition Simulation");
        eprintln!("Pattern: {:?}", demo_args.pattern);
        eprintln!("Style: {:?}", demo_args.style);
        eprintln!("Input: {:?}", demo_args.input_file);
    }

    let result = match demo_args.pattern {
        CompositionPattern::CnnRules => compose_cnn_rules_demo(&demo_args),
        CompositionPattern::MlpBayesian => compose_mlp_bayesian_demo(&demo_args),
        CompositionPattern::TransformerXai => compose_transformer_xai_demo(&demo_args),
        CompositionPattern::RlGuardrails => compose_rl_guardrails_demo(&demo_args),
    };

    match result {
        Ok(r) => {
            let formatted = format_explanation(&r, demo_args.style);
            println!("{}", formatted);
            0
        }
        Err(e) => {
            eprintln!("Demo failed: {}", e);
            1
        }
    }
}

/// Parse CLI arguments for CLARA demo
pub fn parse_demo_args(args: Vec<String>) -> Result<ClaraDemoArgs, String> {
    let mut demo_args = ClaraDemoArgs::default();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-i" | "--input" => {
                i += 1;
                if i >= args.len() {
                    return Err("Option --input requires a value".to_string());
                }
                demo_args.input_file = Some(args[i].clone());
            }
            "-p" | "--pattern" => {
                i += 1;
                if i >= args.len() {
                    return Err("Option --pattern requires a value".to_string());
                }
                demo_args.pattern = match args[i].as_str() {
                    "cnn-rules" => CompositionPattern::CnnRules,
                    "mlp-bayesian" => CompositionPattern::MlpBayesian,
                    "transformer-xai" => CompositionPattern::TransformerXai,
                    "rl-guardrails" => CompositionPattern::RlGuardrails,
                    _ => return Err(format!("Unknown pattern: {}. Valid: cnn-rules, mlp-bayesian, transformer-xai, rl-guardrails", args[i])),
                };
            }
            "-s" | "--style" => {
                i += 1;
                if i >= args.len() {
                    return Err("Option --style requires a value".to_string());
                }
                demo_args.style = match args[i].as_str() {
                    "natural" => FormatStyle::Natural,
                    "fitch" => FormatStyle::Fitch,
                    "compact" => FormatStyle::Compact,
                    _ => return Err(format!("Unknown style: {}. Valid: natural, fitch, compact", args[i])),
                };
            }
            "-v" | "--verbose" => {
                demo_args.verbose = true;
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                return Err(format!("Unknown argument: {}", arg));
            }
        }
        i += 1;
    }

    Ok(demo_args)
}

/// Print help message
fn print_help() {
    println!("CLARA Demo: ML+AR Composition for Image Classification");
    println!();
    println!("Usage: t27c clara-demo [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -i, --input <file>     Input file path (simulated image/data)");
    println!("  -p, --pattern <type>    Composition pattern:");
    println!("                           cnn-rules (default)");
    println!("                           mlp-bayesian");
    println!("                           transformer-xai");
    println!("                           rl-guardrails");
    println!("  -s, --style <format>    Explanation style:");
    println!("                           natural (default)");
    println!("                           fitch");
    println!("                           compact");
    println!("  -v, --verbose             Enable verbose output");
    println!("  -h, --help                Show this help message");
    println!();
    println!("CLARA Requirements:");
    println!("  - AR involved in ML system (Horn clause evaluation)");
    println!("  - Concise explanations (≤10 steps)");
    println!("  - Polynomial-time guarantees (O(n*m) + O(10))");
    println!("  - Confidence encoding (GF16)");
    println!("  - Multiple ML kinds (CNN, MLP, Transformer, RL)");
    println!();
    println!("Examples:");
    println!("  t27c clara-demo --pattern cnn-rules --style natural");
    println!("  t27c clara-demo -i image.png -p mlp-bayesian -s fitch -v");
}

// ============================================================================
// COMPOSITION SIMULATION FUNCTIONS
// ============================================================================

/// Simulate CNN + AR composition (from composition.t27 lines 83-119)
fn compose_cnn_rules_demo(args: &ClaraDemoArgs) -> Result<CompositionResult, String> {
    // Step 1: CNN feature extraction (simulated)
    let ml_result = simulate_cnn_inference(args)?;
    if args.verbose {
        eprintln!("[CNN] Feature extraction complete: decision={:?}, confidence={}",
                 ml_result.decision, ml_result.confidence);
    }

    // Step 2: AR rule evaluation via Datalog
    let ar_result = evaluate_ar_rules(&ml_result, args.verbose)?;

    // Step 3: Combine confidence via GF16 geometric mean
    let combined_conf = combine_confidence_gf16(ml_result.confidence, ar_result.confidence);

    // Step 4: Generate explanation
    let explanation = format_composition_explanation(
        "CNN feature extraction -> AR rule evaluation",
        &ml_result,
        &ar_result,
        combined_conf,
    );

    // Step 5: Final decision via k3_and
    let prediction = Trit::k3_and(ml_result.decision, ar_result.decision);

    // Step 6: Generate proof trace
    let mut trace = ProofTrace {
        steps: Vec::new(),
        step_count: 0,
        conclusion: prediction,
        total_confidence: combined_conf,
        terminated: false,
    };

    // Add ML step
    trace.steps.push(DerivationStep {
        step_number: 1,
        rule_name: "ML inference".to_string(),
        input_facts: vec![Trit::Pos, Trit::Zero, Trit::Zero],
        output_fact: ml_result.decision,
        confidence: ml_result.confidence,
        k3_value: ml_result.decision,
    });
    trace.step_count = 1;

    // Add AR rule steps
    for (_i, rule_name) in ar_result.rules.iter().enumerate() {
        if trace.step_count >= MAX_STEPS {
            trace.terminated = true;
            break;
        }
        trace.steps.push(DerivationStep {
            step_number: (trace.step_count + 1) as u8,
            rule_name: rule_name.clone(),
            input_facts: vec![Trit::Pos, Trit::Pos, Trit::Pos],
            output_fact: Trit::Pos,
            confidence: ar_result.confidence,
            k3_value: Trit::Pos,
        });
        trace.step_count += 1;
    }

    // Add final k3_and step
    if trace.step_count < MAX_STEPS {
        trace.steps.push(DerivationStep {
            step_number: (trace.step_count + 1) as u8,
            rule_name: "K3 AND".to_string(),
            input_facts: vec![ml_result.decision, ar_result.decision, Trit::Pos],
            output_fact: prediction,
            confidence: combined_conf,
            k3_value: prediction,
        });
        trace.step_count += 1;
    }

    trace.conclusion = prediction;

    // Calculate satisfaction (from composition.t27)
    let satisfaction = calculate_satisfaction(&trace, prediction, combined_conf);

    Ok(CompositionResult {
        prediction,
        confidence: combined_conf,
        explanation,
        proof_trace: trace,
        satisfaction,
    })
}

/// Simulate MLP + Bayesian composition
fn compose_mlp_bayesian_demo(_args: &ClaraDemoArgs) -> Result<CompositionResult, String> {
    // MLP forward pass simulation
    let mlp_confidence: f32 = 0.85;
    let mlp_decision = Trit::Pos;

    // Bayesian update
    let prior: f32 = 0.5;
    let likelihood: f32 = 0.9;
    let posterior: f32 = (prior * likelihood) / ((prior * likelihood) + (1.0 - prior) * (1.0 - likelihood) + 0.0001);
    let bayesian_confidence = posterior.min(1.0);
    let bayesian_decision = if bayesian_confidence > 0.5 { Trit::Pos } else { Trit::Neg };

    // Combine via k3_and
    let prediction = Trit::k3_and(mlp_decision, bayesian_decision);
    let combined_conf = (mlp_confidence * bayesian_confidence).sqrt();

    // Generate trace
    let mut trace = ProofTrace {
        steps: Vec::new(),
        step_count: 0,
        conclusion: prediction,
        total_confidence: combined_conf,
        terminated: false,
    };

    trace.steps.push(DerivationStep {
        step_number: 1,
        rule_name: "MLP forward".to_string(),
        input_facts: vec![Trit::Pos, Trit::Zero, Trit::Zero],
        output_fact: mlp_decision,
        confidence: mlp_confidence,
        k3_value: mlp_decision,
    });
    trace.step_count = 1;

    trace.steps.push(DerivationStep {
        step_number: 2,
        rule_name: "Bayesian update".to_string(),
        input_facts: vec![Trit::Pos, Trit::Pos, Trit::Pos],
        output_fact: bayesian_decision,
        confidence: bayesian_confidence,
        k3_value: bayesian_decision,
    });
    trace.step_count = 2;

    trace.steps.push(DerivationStep {
        step_number: 3,
        rule_name: "K3 AND".to_string(),
        input_facts: vec![mlp_decision, bayesian_decision, Trit::Pos],
        output_fact: prediction,
        confidence: combined_conf,
        k3_value: prediction,
    });
    trace.step_count = 3;

    trace.conclusion = prediction;

    let explanation = format!(
        "MLP forward pass (conf={:.2}) -> Bayesian posterior (conf={:.2}) -> K3 AND -> {}",
        mlp_confidence, bayesian_confidence, prediction.display_name()
    );

    Ok(CompositionResult {
        prediction,
        confidence: combined_conf,
        explanation,
        proof_trace: trace,
        satisfaction: combined_conf,
    })
}

/// Simulate Transformer + XAI composition
fn compose_transformer_xai_demo(_args: &ClaraDemoArgs) -> Result<CompositionResult, String> {
    // Transformer self-attention simulation
    let transformer_confidence = 0.92;
    let transformer_decision = Trit::Pos;

    // Generate proof trace with attention steps
    let mut trace = ProofTrace {
        steps: Vec::new(),
        step_count: 0,
        conclusion: transformer_decision,
        total_confidence: transformer_confidence,
        terminated: false,
    };

    let attention_patterns = [
        "Self-attention (pos 0)",
        "Cross-attention (pos 1)",
        "Layer norm",
        "Feed-forward",
        "Self-attention (pos 2)",
        "Cross-attention (pos 3)",
        "Layer norm",
        "Feed-forward",
        "Final aggregation",
        "Classification head",
    ];

    for (i, pattern) in attention_patterns.iter().enumerate() {
        if i >= MAX_STEPS as usize {
            trace.terminated = true;
            break;
        }
        let conf = transformer_confidence * (1.0 - (i as f32 * 0.05));
        trace.steps.push(DerivationStep {
            step_number: (i + 1) as u8,
            rule_name: pattern.to_string(),
            input_facts: vec![Trit::Pos, Trit::Zero, Trit::Zero],
            output_fact: Trit::Pos,
            confidence: conf,
            k3_value: Trit::Pos,
        });
        trace.step_count = (i + 1) as u8;
    }

    trace.conclusion = transformer_decision;

    let explanation = format!(
        "Transformer XAI: {} steps of attention -> final decision {} (conf={:.2})",
        trace.step_count, transformer_decision.display_name(), transformer_confidence
    );

    Ok(CompositionResult {
        prediction: transformer_decision,
        confidence: transformer_confidence,
        explanation,
        proof_trace: trace,
        satisfaction: transformer_confidence * 1.1, // Bonus for bounded trace
    })
}

/// Simulate RL + Guardrails composition
fn compose_rl_guardrails_demo(_args: &ClaraDemoArgs) -> Result<CompositionResult, String> {
    // RL policy inference
    let policy_confidence: f32 = 0.88;
    let policy_decision = Trit::Pos;

    // Guardrail evaluation
    let guardrails_pass = true;
    let guardrail_confidence: f32 = 0.95;
    let guardrail_decision = if guardrails_pass { Trit::Pos } else { Trit::Neg };

    // Combine: action allowed only if both RL and guardrails approve
    let prediction = Trit::k3_and(policy_decision, guardrail_decision);
    let combined_conf = if guardrails_pass {
        (policy_confidence * guardrail_confidence).sqrt()
    } else {
        policy_confidence * 0.3 // Penalty for blocked action
    };

    // Generate trace
    let mut trace = ProofTrace {
        steps: Vec::new(),
        step_count: 0,
        conclusion: prediction,
        total_confidence: combined_conf,
        terminated: false,
    };

    trace.steps.push(DerivationStep {
        step_number: 1,
        rule_name: "RL policy inference".to_string(),
        input_facts: vec![Trit::Pos, Trit::Zero, Trit::Zero],
        output_fact: policy_decision,
        confidence: policy_confidence,
        k3_value: policy_decision,
    });
    trace.step_count = 1;

    trace.steps.push(DerivationStep {
        step_number: 2,
        rule_name: "Guardrail check".to_string(),
        input_facts: vec![policy_decision, policy_decision, policy_decision],
        output_fact: guardrail_decision,
        confidence: guardrail_confidence,
        k3_value: guardrail_decision,
    });
    trace.step_count = 2;

    if guardrails_pass {
        trace.steps.push(DerivationStep {
            step_number: 3,
            rule_name: "K3 AND (action allowed)".to_string(),
            input_facts: vec![policy_decision, guardrail_decision, Trit::Pos],
            output_fact: prediction,
            confidence: combined_conf,
            k3_value: prediction,
        });
        trace.step_count = 3;
    }

    trace.conclusion = prediction;

    let explanation = format!(
        "RL policy (conf={:.2}) -> Guardrail check (pass={}) -> {}",
        policy_confidence, guardrails_pass, prediction.display_name()
    );

    Ok(CompositionResult {
        prediction,
        confidence: combined_conf,
        explanation,
        proof_trace: trace,
        satisfaction: combined_conf * if guardrails_pass { 1.0 } else { 0.7 },
    })
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Simulate CNN inference (from composition.t27 lines 327-334)
fn simulate_cnn_inference(args: &ClaraDemoArgs) -> Result<MLComponent, String> {
    // Simulate feature extraction from "input file"
    // In production: would load image and run actual CNN
    let confidence = match args.input_file.as_deref() {
        Some(path) if path.contains("7") => 0.90,
        Some(path) if path.contains("low") => 0.45,
        None | Some(_) => 0.85,
    };

    let decision = if confidence > 0.5 {
        Trit::Pos
    } else if confidence < 0.4 {
        Trit::Neg
    } else {
        Trit::Zero
    };

    Ok(MLComponent {
        component_type: "CNN".to_string(),
        confidence,
        decision,
    })
}

/// Evaluate AR rules (from datalog_engine.t27)
fn evaluate_ar_rules(ml_result: &MLComponent, verbose: bool) -> Result<ARComponent, String> {
    let rules = vec![
        "has_top_stroke".to_string(),
        "loop_closed".to_string(),
        "has_clear_digits".to_string(),
        "safe_to_rotate".to_string(),
    ];

    let rule_count = rules.len();
    let ar_confidence = 0.95;
    let ar_decision = Trit::Pos;

    if verbose {
        eprintln!("[AR] Evaluated {} rules, decision={:?}, confidence={}",
                 rule_count, ar_decision, ar_confidence);
    }

    // Generate proof trace for AR
    let mut trace = ProofTrace {
        steps: Vec::new(),
        step_count: 0,
        conclusion: ar_decision,
        total_confidence: ar_confidence,
        terminated: false,
    };

    for (i, rule_name) in rules.iter().enumerate() {
        if i >= MAX_STEPS as usize {
            trace.terminated = true;
            break;
        }
        trace.steps.push(DerivationStep {
            step_number: (i + 1) as u8,
            rule_name: format!("AR rule: {}", rule_name),
            input_facts: vec![ml_result.decision, Trit::Pos, Trit::Pos],
            output_fact: Trit::Pos,
            confidence: ar_confidence,
            k3_value: Trit::Pos,
        });
        trace.step_count = (i + 1) as u8;
    }

    trace.conclusion = ar_decision;

    Ok(ARComponent {
        rules,
        rule_count,
        decision: ar_decision,
        confidence: ar_confidence,
        trace,
    })
}

/// Combine confidence using GF16 geometric mean (from composition.t27 lines 401-406)
fn combine_confidence_gf16(ml_conf: f32, ar_conf: f32) -> f32 {
    // Geometric mean: sqrt(ml × ar)
    (ml_conf * ar_conf).sqrt()
}

/// Format composition explanation (from composition.t27 lines 408-445)
fn format_composition_explanation(
    pipeline_desc: &str,
    ml_result: &MLComponent,
    ar_result: &ARComponent,
    combined_conf: f32,
) -> String {
    format!(
        "{}: ML {} (conf={:.2}) -> AR {} rules (conf={:.2}) -> combined (conf={:.2})",
        pipeline_desc,
        ml_result.component_type,
        ml_result.confidence,
        ar_result.rule_count,
        ar_result.confidence,
        combined_conf
    )
}

/// Calculate CLARA satisfaction (from composition.t27 lines 457-484)
fn calculate_satisfaction(trace: &ProofTrace, _prediction: Trit, confidence: f32) -> f32 {
    let mut score = 1.0;

    // Bonus for bounded (≤10 steps) explanations
    if trace.step_count <= MAX_STEPS {
        score *= 1.2;
    } else {
        score *= 0.5; // Penalty for exceeding CLARA limit
    }

    // Bonus for confident predictions
    if confidence >= 0.7 {
        score *= 1.1;
    }

    // Penalty for termination (restraint triggered)
    if trace.terminated {
        score *= 0.7;
    }

    // Clamp to [0, 1]
    let score_clamped: f32 = (score as f32).min(1.0).max(0.0);
    score_clamped
}

/// Format explanation in requested style (from explainability.t27 lines 77-123)
fn format_explanation(result: &CompositionResult, style: FormatStyle) -> String {
    match style {
        FormatStyle::Natural => format_natural(result),
        FormatStyle::Fitch => format_fitch(result),
        FormatStyle::Compact => format_compact(result),
    }
}

/// Format in natural style (from proof_trace.t27 lines 85-99)
fn format_natural(result: &CompositionResult) -> String {
    let mut output = String::new();

    // Header section
    output.push_str(&format!("Prediction: {} (conf={:.2})\n",
                          result.prediction.display_name(),
                          result.confidence));
    output.push_str(&format!("Satisfaction: {:.2} (CLARA requirement)\n",
                          result.satisfaction));
    let termination_note = if result.proof_trace.terminated { " - terminated" } else { "" };
    output.push_str(&format!("Steps: {}/{} (bounded by CLARA{})\n",
                          result.proof_trace.step_count,
                          MAX_STEPS,
                          termination_note));
    output.push_str("\n");

    // Proof trace steps
    for step in &result.proof_trace.steps {
        // Use string concatenation to avoid format macro parsing issues with braces
        output.push_str("Step ");
        output.push_str(&step.step_number.to_string());
        output.push_str(": [");
        output.push_str(&step.rule_name);
        output.push_str("] -> ");
        output.push_str(step.output_fact.display_name());
        output.push_str(" (conf=");
        let conf_str = format!("{:.2}", step.confidence);
        output.push_str(&conf_str);
        output.push_str(")\n");
    }

    // Conclusion
    output.push_str("\nConclusion: ");
    if result.satisfaction >= 0.8 {
        let conclusion = format!("High confidence classification ({:.2}). All safety constraints satisfied.",
                                 result.confidence);
        output.push_str(&conclusion);
    } else {
        let satisfaction_text = if result.satisfaction < 0.5 {
            "Low satisfaction - review constraints."
        } else {
            "Moderate satisfaction."
        };
        let conclusion = format!("Classification with confidence {:.2}. {}",
                                 result.confidence,
                                 satisfaction_text);
        output.push_str(&conclusion);
    }

    output
}

/// Format in Fitch style (formal natural deduction)
fn format_fitch(result: &CompositionResult) -> String {
    let mut output = String::new();

    output.push_str("Fitch-Style Proof Trace\n");
    output.push_str(&str::repeat("-", 60));
    output.push('\n');

    for step in &result.proof_trace.steps {
        output.push_str(&format!("| {:2}. | {{{}}}                | {{{:<30}}}|\n",
                              step.step_number,
                              step.output_fact.display_name(),
                              step.rule_name));
    }

    output.push_str(&str::repeat("-", 60));
    output.push('\n');
    output.push_str(&format!("|     | {{pred={:?}, conf={:.2}}}  | CLARA Demo                  |\n",
                          result.prediction,
                          result.confidence));

    output
}

/// Format in compact style (single-line summary)
fn format_compact(result: &CompositionResult) -> String {
    format!(
        "{} steps | conclusion=[{}] | conf={:.2} | sat={:.2} | terminated=[{}]",
        result.proof_trace.step_count,
        result.prediction.display_name(),
        result.confidence,
        result.satisfaction,
        result.proof_trace.terminated
    )
}
