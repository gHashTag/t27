import { QueenTick } from './queen-tick';

/**
 * Queen's review system for handling branches and their diffs
 * 
 * This module implements the Queen's review process where branches progress
 * through states: imported -> ready -> review -> review-done -> done
 * 
 * The issue: "could not read the diff of finished work" is logged 1105 times
 * in 46 minutes without identifying which specific branch failed or why.
 * 
 * Goal: Every finished branch reaches a review round; the diff read either
 * succeeds or reports why, once per branch.
 */

export interface Branch {
  id: string;
  hash: string;
  state: 'imported' | 'ready' | 'review' | 'review-done' | 'done';
  importedAt?: Date;
  reviewedAt?: Date;
  diffReadError?: string;
  diffReadSuccess?: boolean;
}

export interface ReviewRound {
  id: string;
  timestamp: Date;
  branchesReviewed: string[];
  branchesWithDiffErrors: string[];
  branchesWithDiffSuccess: string[];
}

export class QueenReviewSystem {
  private branches: Map<string, Branch> = new Map();
  private reviewRounds: ReviewRound[] = [];
  private diffReadAttempts: Map<string, number> = new Map();
  private maxDiffReadAttempts = 3;

  /**
   * Import a new branch into the system
   */
  importBranch(branchId: string, hash: string): void {
    const branch: Branch = {
      id: branchId,
      hash: hash,
      state: 'imported',
      importedAt: new Date()
    };
    
    this.branches.set(branchId, branch);
    console.log(`[${new Date().toISOString()}] Branch imported: ${branchId} (${hash})`);
  }

  /**
   * Move branch to ready state
   */
  markBranchReady(branchId: string): void {
    const branch = this.branches.get(branchId);
    if (!branch) {
      throw new Error(`Branch ${branchId} not found`);
    }
    
    branch.state = 'ready';
    console.log(`[${new Date().toISOString()}] Branch ready: ${branchId}`);
  }

  /**
   * Attempt to read diff for a branch in review state
   * This is where the issue occurs - we need to track failures with specific causes
   */
  async readBranchDiff(branchId: string): Promise<{ success: boolean; error?: string; diff?: string }> {
    const branch = this.branches.get(branchId);
    if (!branch) {
      throw new Error(`Branch ${branchId} not found`);
    }

    // Track attempts to prevent infinite retry loops
    const attempts = this.diffReadAttempts.get(branchId) || 0;
    this.diffReadAttempts.set(branchId, attempts + 1);

    try {
      // Simulate diff reading - in real implementation this would call git or similar
      const diff = await this.fetchBranchDiff(branchId);
      
      branch.diffReadSuccess = true;
      branch.diffReadError = undefined;
      branch.state = 'review-done';
      branch.reviewedAt = new Date();
      
      console.log(`[${new Date().toISOString()}] Diff read SUCCESS: ${branchId}`);
      return { success: true, diff };
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      branch.diffReadError = errorMessage;
      branch.diffReadSuccess = false;
      
      // Log the specific error with branch identification
      console.error(`[${new Date().toISOString()}] Diff read FAILED for ${branchId}: ${errorMessage}`);
      
      // If max attempts reached, move to review-done to prevent infinite loops
      if (attempts >= this.maxDiffReadAttempts) {
        branch.state = 'review-done';
        branch.reviewedAt = new Date();
        console.log(`[${new Date().toISOString()}] Max attempts reached for ${branchId}, marking as review-done`);
      }
      
      return { success: false, error: errorMessage };
    }
  }

  /**
   * Create a review round and process branches
   */
  async createReviewRound(): Promise<ReviewRound> {
    const round: ReviewRound = {
      id: `round-${Date.now()}`,
      timestamp: new Date(),
      branchesReviewed: [],
      branchesWithDiffErrors: [],
      branchesWithDiffSuccess: []
    };

    console.log(`[${new Date().toISOString()}] Creating review round: ${round.id}`);

    // Find branches in 'ready' state that need review
    const branchesToReview = Array.from(this.branches.values())
      .filter(branch => branch.state === 'ready');

    for (const branch of branchesToReview) {
      try {
        // Move branch to review state
        branch.state = 'review';
        console.log(`[${new Date().toISOString()}] Starting review for ${branch.id}`);

        // Attempt to read diff
        const result = await this.readBranchDiff(branch.id);
        
        round.branchesReviewed.push(branch.id);
        
        if (result.success) {
          round.branchesWithDiffSuccess.push(branch.id);
        } else {
          round.branchesWithDiffErrors.push(branch.id);
        }
        
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Unknown error';
        console.error(`[${new Date().toISOString()}] Review FAILED for ${branch.id}: ${errorMessage}`);
        round.branchesReviewed.push(branch.id);
        round.branchesWithDiffErrors.push(branch.id);
      }
    }

    this.reviewRounds.push(round);
    return round;
  }

  /**
   * Get statistics about the review system
   */
  getReviewStats(): {
    totalBranches: number;
    branchesByState: Record<string, number>;
    totalReviewRounds: number;
    branchesWithDiffErrors: string[];
    branchesWithDiffSuccess: string[];
    recentFailures: Array<{ branchId: string; error: string; timestamp: Date }>;
  } {
    const branches = Array.from(this.branches.values());
    const branchesByState = branches.reduce((acc, branch) => {
      acc[branch.state] = (acc[branch.state] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    const recentFailures = branches
      .filter(branch => branch.diffReadError)
      .map(branch => ({
        branchId: branch.id,
        error: branch.diffReadError!,
        timestamp: branch.reviewedAt || new Date()
      }))
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
      .slice(0, 10);

    return {
      totalBranches: branches.length,
      branchesByState,
      totalReviewRounds: this.reviewRounds.length,
      branchesWithDiffErrors: branches.filter(b => b.diffReadError === false).map(b => b.id),
      branchesWithDiffSuccess: branches.filter(b => b.diffReadSuccess === true).map(b => b.id),
      recentFailures
    };
  }

  /**
   * Check if a specific branch has been reviewed
   */
  isBranchReviewed(branchId: string): boolean {
    const branch = this.branches.get(branchId);
    return branch ? branch.state === 'review-done' || branch.state === 'done' : false;
  }

  /**
   * Simulate fetching branch diff (would be actual git operation in real implementation)
   */
  private async fetchBranchDiff(branchId: string): Promise<string> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 100));
    
    // Simulate occasional failures (this is where the real issue would manifest)
    if (Math.random() < 0.1) { // 10% chance of failure
      throw new Error(`Failed to read diff for branch ${branchId}: repository not accessible`);
    }
    
    // Simulate successful diff
    return `diff --git a/src/file.ts b/src/file.ts\nnew file mode 100644\nindex 0000000..1234567\n--- /dev/null\n+++ b/src/file.ts\n@@ -0,0 +1,10 @@\n+export function example() {\n+  return 'Hello from ${branchId}';\n+}\n`;
  }
}

// Export singleton instance for use by the Queen's system
export const queenReviewSystem = new QueenReviewSystem();

/**
 * Main function to run the Queen's review tick
 * This is the entry point that would be called by the Queen's scheduler
 */
export async function runQueenTick(): Promise<void> {
  console.log(`[${new Date().toISOString()}] Starting Queen review tick`);
  
  try {
    // Create a new review round
    const round = await queenReviewSystem.createReviewRound();
    
    // Log round results
    console.log(`[${new Date().toISOString()}] Review round completed: ${round.id}`);
    console.log(`  - Branches reviewed: ${round.branchesReviewed.length}`);
    console.log(`  - Diff reads successful: ${round.branchesWithDiffSuccess.length}`);
    console.log(`  - Diff reads failed: ${round.branchesWithDiffErrors.length}`);
    
    // Get and log statistics
    const stats = queenReviewSystem.getReviewStats();
    console.log(`[${new Date().toISOString()}] Review system statistics:`);
    console.log(`  - Total branches: ${stats.totalBranches}`);
    console.log(`  - Total review rounds: ${stats.totalReviewRounds}`);
    console.log(`  - Branches by state:`, stats.branchesByState);
    
    if (stats.recentFailures.length > 0) {
      console.log(`[${new Date().toISOString()}] Recent diff read failures:`);
      stats.recentFailures.forEach(failure => {
        console.log(`  - ${failure.branchId}: ${failure.error} (${failure.timestamp.toISOString()})`);
      });
    }
    
  } catch (error) {
    console.error(`[${new Date().toISOString()}] Queen tick failed:`, error);
    throw error;
  }
}

// Example usage for testing
if (require.main === module) {
  runQueenTick().catch(console.error);
}