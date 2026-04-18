#!/usr/bin/env python3
"""
Example 3: Autonomous Driving with RL + VSA + Safety Constraints
================================================================

Composition: RL Policy Network → VSA Encoding → Rule Engine → Guardrails

This example demonstrates autonomous driving decision-making with:
1. RL policy network for action selection
2. VSA encoding for state-action representation
3. Rule engine for safety constraint checking
4. Guardrails for allow/block decisions

Safety-critical system with bounded rationality and explicit safety checks.

Author: Dmitrii Vasilev (T27 Project)
SPDX-License-Identifier: Apache-2.0
"""

from dataclasses import dataclass
from typing import List, Tuple, Dict, Optional
import math


# ============================================================================
# T27 Ternary Types
# ============================================================================

TRIT_NEG = -1
TRIT_ZERO = 0
TRIT_POS = 1

Trit = int

VSA_DIM = 1024


# ============================================================================
# VSA Operations (from specs/vsa/ops.t27)
# ============================================================================

def to_trits(values: List[float], dim: int = VSA_DIM) -> List[Trit]:
    """Convert float vector to ternary hypervector."""
    trits = []
    for v in values[:dim]:
        if v > 0.33:
            trits.append(TRIT_POS)
        elif v < -0.33:
            trits.append(TRIT_NEG)
        else:
            trits.append(TRIT_ZERO)
    while len(trits) < dim:
        trits.append(TRIT_ZERO)
    return trits[:dim]


def bind(a: List[Trit], b: List[Trit], length: int) -> List[Trit]:
    """Bind operation (XOR-like) for associative memory."""
    result = []
    for i in range(length):
        ai, bi = a[i], b[i]
        if ai == TRIT_ZERO:
            result.append(bi)
        elif bi == TRIT_ZERO:
            result.append(ai)
        else:
            result.append(TRIT_POS if ai == bi else TRIT_NEG)
    return result


def unbind(bound: List[Trit], key: List[Trit], length: int) -> List[Trit]:
    """Unbind operation (same as bind for XOR-like binding)."""
    return bind(bound, key, length)


def cosine_similarity(a: List[Trit], b: List[Trit], length: int) -> float:
    """Cosine similarity for hypervector comparison."""
    dot = sum(a[i] * b[i] for i in range(length))
    norm_a = math.sqrt(sum(1 for i in range(length) if a[i] != TRIT_ZERO))
    norm_b = math.sqrt(sum(1 for i in range(length) if b[i] != TRIT_ZERO))
    if norm_a == 0.0 or norm_b == 0.0:
        return 0.0
    return dot / (norm_a * norm_b)


# ============================================================================
# RL Policy Network (simulated)
# ============================================================================

Action = str


@dataclass
class RLState:
    """State representation for RL policy network."""
    ego_velocity: float  # m/s
    ego_acceleration: float  # m/s²
    distance_to_front: float  # m
    front_vehicle_velocity: float  # m/s
    distance_to_rear: float  # m
    rear_vehicle_velocity: float  # m/s
    lane_position: float  # -1 (left), 0 (center), +1 (right)
    road_curvature: float  # 1/radius
    weather_condition: float  # 0-1 (dry to wet)
    traffic_density: float  # 0-1 (light to heavy)


class RLPolicyNetwork:
    """
    Simulated RL policy network for action selection.

    In real system, this would be a trained neural network.
    """

    def __init__(self):
        self.action_space = [
            "accelerate",
            "maintain_speed",
            "decelerate",
            "change_lane_left",
            "change_lane_right",
            "emergency_brake"
        ]

    def select_action(self, state: RLState) -> Tuple[Action, float]:
        """
        Select action based on state using RL policy.

        Returns: (action, confidence)
        """
        # Simulated policy logic
        if state.distance_to_front < 10:
            return "emergency_brake", 0.98
        elif state.distance_to_front < 20:
            if state.ego_velocity > state.front_vehicle_velocity:
                return "decelerate", 0.85
            return "maintain_speed", 0.7
        elif state.road_curvature > 0.01:
            return "decelerate", 0.6
        elif state.weather_condition > 0.7:
            return "maintain_speed", 0.65
        else:
            return "accelerate", 0.7


# ============================================================================
# Safety Rules (from specs/ar/restraint.t27)
# ============================================================================

@dataclass
class SafetyConstraint:
    name: str
    check_fn: callable
    is_blocking: bool = True


class SafetyRuleEngine:
    """
    Rule engine for safety constraint checking.

    All safety constraints must be satisfied for action to proceed.
    """

    def __init__(self):
        self.constraints = self._load_constraints()

    def _load_constraints(self) -> List[SafetyConstraint]:
        """Load safety constraints for autonomous driving."""
        return [
            SafetyConstraint(
                name="minimum_following_distance",
                check_fn=lambda s, a: self._check_following_distance(s, a),
                is_blocking=True
            ),
            SafetyConstraint(
                name="safe_lane_change",
                check_fn=lambda s, a: self._check_lane_change(s, a),
                is_blocking=True
            ),
            SafetyConstraint(
                name="velocity_limit",
                check_fn=lambda s, a: self._check_velocity_limit(s, a),
                is_blocking=True
            ),
            SafetyConstraint(
                name="weather_adaptation",
                check_fn=lambda s, a: self._check_weather_adaptation(s, a),
                is_blocking=False  # Warning only
            ),
        ]

    def _check_following_distance(self, state: RLState, action: Action) -> bool:
        """Ensure safe following distance."""
        if action == "accelerate":
            # 2-second rule minimum
            min_distance = state.ego_velocity * 2.0
            return state.distance_to_front >= min_distance
        return True

    def _check_lane_change(self, state: RLState, action: Action) -> bool:
        """Ensure lane change is safe."""
        if "lane" in action:
            # Check rear vehicle distance
            return state.distance_to_rear > 15
        return True

    def _check_velocity_limit(self, state: RLState, action: Action) -> bool:
        """Ensure velocity is within safe limits."""
        speed_limit = 30.0  # m/s (~108 km/h)
        if action == "accelerate":
            return state.ego_velocity < speed_limit
        return True

    def _check_weather_adaptation(self, state: RLState, action: Action) -> bool:
        """Warn about weather adaptation."""
        if state.weather_condition > 0.7 and action == "accelerate":
            return False  # Warning: reduce speed in wet conditions
        return True

    def check_constraints(self, state: RLState, action: Action) -> Tuple[bool, List[str]]:
        """
        Check all safety constraints.

        Returns: (all_satisfied, list_of_failed_constraints)
        """
        failed = []
        warnings = []

        for constraint in self.constraints:
            if not constraint.check_fn(state, action):
                if constraint.is_blocking:
                    failed.append(constraint.name)
                else:
                    warnings.append(constraint.name)

        all_satisfied = len(failed) == 0
        return all_satisfied, failed


# ============================================================================
# VSA State Encoding (from specs/vsa/ops.t27)
# ============================================================================

class VSAStateEncoder:
    """
    Encode RL state-action pairs to hypervectors.

    Used for semantic memory and experience replay.
    """

    def encode_state(self, state: RLState) -> List[Trit]:
        """Encode state to hypervector."""
        values = [
            state.ego_velocity / 50.0,  # Normalize
            state.ego_acceleration / 10.0,
            state.distance_to_front / 100.0,
            state.front_vehicle_velocity / 50.0,
            state.distance_to_rear / 100.0,
            state.rear_vehicle_velocity / 50.0,
            state.lane_position,
            state.road_curvature * 100,
            state.weather_condition,
            state.traffic_density,
        ]
        # Pad to VSA_DIM
        while len(values) < VSA_DIM:
            values.append(0.0)
        return to_trits(values, VSA_DIM)

    def encode_action(self, action: Action) -> List[Trit]:
        """Encode action to hypervector."""
        action_values = [hash(action) % 100 / 100.0]
        while len(action_values) < VSA_DIM:
            action_values.append(0.0)
        return to_trits(action_values, VSA_DIM)

    def bind_state_action(self, state: RLState, action: Action) -> List[Trit]:
        """Bind state and action for associative memory."""
        state_hv = self.encode_state(state)
        action_hv = self.encode_action(action)
        return bind(state_hv, action_hv, VSA_DIM)


# ============================================================================
# Guardrails (from specs/ar/composition.t27)
# ============================================================================

class Guardrails:
    """
    Guardrails system for final allow/block decision.

    Safety-critical: blocks unsafe actions regardless of RL confidence.
    """

    def __init__(self, safety_engine: SafetyRuleEngine):
        self.safety_engine = safety_engine
        self.emergency_brake_threshold = 0.3  # Distance in meters

    def allow_or_block(self, state: RLState, action: Action,
                       rl_confidence: float) -> Tuple[bool, str]:
        """
        Make final allow/block decision.

        Returns: (allowed, reason)
        """
        # Emergency check
        if state.distance_to_front < self.emergency_brake_threshold:
            if action != "emergency_brake":
                return False, "EMERGENCY: Front vehicle too close"

        # Safety constraints
        safe, failed = self.safety_engine.check_constraints(state, action)
        if not safe:
            return False, f"BLOCKED: {', '.join(failed)}"

        # Allow action
        return True, f"ALLOWED: {action} (confidence: {rl_confidence:.2f})"


# ============================================================================
# Autonomous Driving System (Composition)
# ============================================================================

class AutonomousDrivingSystem:
    """
    Complete autonomous driving pipeline.

    Composition: RL → VSA → Rules → Guardrails
    """

    def __init__(self):
        self.rl_policy = RLPolicyNetwork()
        self.safety_engine = SafetyRuleEngine()
        self.vsa_encoder = VSAStateEncoder()
        self.guardrails = Guardrails(self.safety_engine)

        # Experience memory (VSA hypervectors)
        self.experience_memory: List[Tuple[List[Trit], RLState, Action]] = []

    def decide(self, state: RLState) -> Dict:
        """
        Make driving decision with full safety pipeline.

        Pipeline:
        1. RL policy selects action
        2. VSA encodes state-action pair
        3. Rule engine checks safety constraints
        4. Guardrails makes final allow/block decision

        Returns: Decision dict with action, allowed, reason, confidence
        """
        # Step 1: RL policy
        action, rl_confidence = self.rl_policy.select_action(state)

        # Step 2: VSA encoding (for experience memory)
        state_action_hv = self.vsa_encoder.bind_state_action(state, action)

        # Step 3: Safety rule checking
        safe, failed_constraints = self.safety_engine.check_constraints(
            state, action
        )

        # Step 4: Guardrails decision
        allowed, reason = self.guardrails.allow_or_block(
            state, action, rl_confidence
        )

        # Store experience
        self.experience_memory.append((state_action_hv, state, action))

        return {
            "state_summary": self._summarize_state(state),
            "rl_action": action,
            "rl_confidence": rl_confidence,
            "safety_constraints_satisfied": safe,
            "failed_constraints": failed_constraints,
            "allowed": allowed,
            "final_reason": reason,
            "vsa_encoded": len(state_action_hv) == VSA_DIM
        }

    def _summarize_state(self, state: RLState) -> str:
        """Summarize state for logging."""
        return (
            f"v={state.ego_velocity:.1f}m/s, "
            f"d_front={state.distance_to_front:.1f}m, "
            f"lane={int(state.lane_position)}, "
            f"weather={state.weather_condition:.1f}"
        )


# ============================================================================
# Main: Example Usage
# ============================================================================

def main():
    """Run the autonomous driving example."""
    print("=" * 60)
    print("Autonomous Driving - RL + VSA + Safety Rules + Guardrails")
    print("=" * 60)
    print()

    # Initialize the system
    ads = AutonomousDrivingSystem()

    # Test scenarios
    scenarios = [
        RLState(
            ego_velocity=20.0,
            ego_acceleration=0.0,
            distance_to_front=50.0,
            front_vehicle_velocity=18.0,
            distance_to_rear=30.0,
            rear_vehicle_velocity=22.0,
            lane_position=0.0,
            road_curvature=0.0,
            weather_condition=0.2,
            traffic_density=0.3
        ),
        RLState(
            ego_velocity=25.0,
            ego_acceleration=1.0,
            distance_to_front=15.0,  # Too close!
            front_vehicle_velocity=20.0,
            distance_to_rear=40.0,
            rear_vehicle_velocity=25.0,
            lane_position=0.0,
            road_curvature=0.0,
            weather_condition=0.5,
            traffic_density=0.5
        ),
        RLState(
            ego_velocity=15.0,
            ego_acceleration=0.0,
            distance_to_front=8.0,  # Emergency!
            front_vehicle_velocity=10.0,
            distance_to_rear=20.0,
            rear_vehicle_velocity=18.0,
            lane_position=0.0,
            road_curvature=0.0,
            weather_condition=0.3,
            traffic_density=0.4
        ),
        RLState(
            ego_velocity=10.0,
            ego_acceleration=-2.0,
            distance_to_front=40.0,
            front_vehicle_velocity=12.0,
            distance_to_rear=5.0,  # Too close for lane change!
            rear_vehicle_velocity=15.0,
            lane_position=0.0,
            road_curvature=0.0,
            weather_condition=0.8,  # Wet conditions
            traffic_density=0.6
        ),
    ]

    for i, state in enumerate(scenarios, 1):
        print("-" * 60)
        print(f"Scenario {i}:")
        print(f"  State: {ads._summarize_state(state)}")
        print()

        decision = ads.decide(state)

        print(f"  RL Action: {decision['rl_action']}")
        print(f"  RL Confidence: {decision['rl_confidence']:.2f}")
        print(f"  Safety Constraints Satisfied: {decision['safety_constraints_satisfied']}")
        if decision['failed_constraints']:
            print(f"  Failed Constraints: {decision['failed_constraints']}")
        print(f"  VSA Encoded: {decision['vsa_encoded']} ({VSA_DIM}-dim)")
        print()
        print(f"  FINAL DECISION: {decision['final_reason']}")
        print()

    print("=" * 60)
    print("Safety-Critical System: All decisions verified by guardrails")
    print("=" * 60)


if __name__ == "__main__":
    main()
