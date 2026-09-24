/**
 * Shared Spec Core Functionality
 * 
 * Core functionality for spec identity validation, source path validation,
 * and spec management across the Queen system.
 */

import { SpecIdentity, SpecAvailability, SpecState, SpecSource } from './specCatalog';

export interface SpecValidationResult {
    isValid: boolean;
    errors: string[];
    warnings: string[];
}

export interface SourcePathValidation {
    exists: boolean;
    isAccessible: boolean;
    isAmbiguous: boolean;
    resolvedPath: string;
    error?: string;
}

/**
 * Validates spec identity including source path validation
 * Addresses missing source path validation for missing/ambiguous paths
 */
export function validateSpecIdentity(specIdentity: SpecIdentity): SpecValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];
    
    // Basic identity validation
    if (!specIdentity.id || specIdentity.id.trim() === '') {
        errors.push('Spec ID is required');
    }
    
    if (!specIdentity.version || specIdentity.version.trim() === '') {
        errors.push('Spec version is required');
    }
    
    if (!specIdentity.repository || specIdentity.repository.trim() === '') {
        errors.push('Repository is required');
    }
    
    // Source path validation - NEW: Check for missing/ambiguous paths
    if (specIdentity.sourcePath) {
        const sourceValidation = validateSourcePath(specIdentity.sourcePath);
        
        if (!sourceValidation.exists) {
            errors.push(`Source path does not exist: ${specIdentity.sourcePath}`);
        }
        
        if (sourceValidation.isAmbiguous) {
            errors.push(`Source path is ambiguous: ${specIdentity.sourcePath}`);
            warnings.push(`Resolved to: ${sourceValidation.resolvedPath}`);
        }
        
        if (!sourceValidation.isAccessible) {
            errors.push(`Source path is not accessible: ${specIdentity.sourcePath}`);
        }
    }
    
    // SHA/hash validation
    if (specIdentity.sha && specIdentity.sha.trim() !== '') {
        if (!isValidSHA(specIdentity.sha)) {
            errors.push(`Invalid SHA format: ${specIdentity.sha}`);
        }
    }
    
    return {
        isValid: errors.length === 0,
        errors,
        warnings
    };
}

/**
 * Validates source path for existence, accessibility, and ambiguity
 * Addresses the missing source path validation requirement
 */
export function validateSourcePath(sourcePath: string): SourcePathValidation {
    try {
        // Check if path exists
        const exists = filesystem.existsSync(sourcePath);
        
        if (!exists) {
            return {
                exists: false,
                isAccessible: false,
                isAmbiguous: false,
                resolvedPath: ''
            };
        }
        
        // Check if accessible
        const isAccessible = filesystem.accessSync(sourcePath, fs.constants.R_OK);
        
        // Check for ambiguity (multiple files with similar names, symlinks, etc.)
        const isAmbiguous = checkPathAmbiguity(sourcePath);
        
        // Resolve to canonical path
        const resolvedPath = path.resolve(sourcePath);
        
        return {
            exists: true,
            isAccessible,
            isAmbiguous,
            resolvedPath
        };
    } catch (error) {
        return {
            exists: false,
            isAccessible: false,
            isAmbiguous: false,
            resolvedPath: '',
            error: error instanceof Error ? error.message : 'Unknown error'
        };
    }
}

/**
 * Checks if a path is ambiguous (symlinks, multiple similar files, etc.)
 */
function checkPathAmbiguity(sourcePath: string): boolean {
    try {
        const stats = fs.lstatSync(sourcePath);
        
        // Check if it's a symlink
        if (stats.isSymbolicLink()) {
            return true;
        }
        
        // Check for multiple files with similar names in the same directory
        const dir = path.dirname(sourcePath);
        const basename = path.basename(sourcePath);
        const files = fs.readdirSync(dir);
        
        const similarFiles = files.filter(file => 
            file !== basename && 
            file.toLowerCase().includes(basename.toLowerCase())
        );
        
        return similarFiles.length > 0;
    } catch {
        return false;
    }
}

/**
 * Validates SHA format
 */
function isValidSHA(sha: string): boolean {
    // SHA-1: 40 chars, SHA-256: 64 chars
    return /^[0-9a-f]{40}$|^[0-9a-f]{64}$/i.test(sha);
}

/**
 * Spec Manager for handling spec lifecycle and state management
 */
export class SpecManager {
    private specs: Map<string, SpecIdentity> = new Map();
    private specStates: Map<string, SpecState> = new Map();
    private frozenSnapshots: Map<string, SpecIdentity> = new Map();
    
    /**
     * Checks if a spec can be rendered as a live update from a frozen snapshot
     * Addresses preventing rendering test fixture with changed issue state from frozen snapshot
     */
    public canRenderLiveUpdate(specId: string, currentSpec: SpecIdentity): boolean {
        const frozenSpec = this.frozenSnapshots.get(specId);
        
        if (!frozenSpec) {
            return true; // No frozen snapshot, allow rendering
        }
        
        // Check if issue state has changed
        if (frozenSpec.issueState !== currentSpec.issueState) {
            return false; // Issue state changed, cannot render from frozen snapshot
        }
        
        // Check if SHA has changed
        if (frozenSpec.sha !== currentSpec.sha) {
            return false; // SHA changed, cannot render from frozen snapshot
        }
        
        return true;
    }
    
    /**
     * Creates a frozen snapshot of a spec
     */
    public createFrozenSnapshot(specId: string, spec: SpecIdentity): void {
        this.frozenSnapshots.set(specId, { ...spec });
    }
    
    /**
     * Validates and registers a new spec
     */
    public registerSpec(spec: SpecIdentity): SpecValidationResult {
        const validation = validateSpecIdentity(spec);
        
        if (validation.isValid) {
            this.specs.set(spec.id, spec);
            this.specStates.set(spec.id, SpecState.Available);
        }
        
        return validation;
    }
    
    /**
     * Gets spec state by ID
     */
    public getSpecState(specId: string): SpecState | undefined {
        return this.specStates.get(specId);
    }
    
    /**
     * Updates spec state
     */
    public updateSpecState(specId: string, state: SpecState): void {
        this.specStates.set(specId, state);
    }
    
    /**
     * Gets all registered specs
     */
    public getAllSpecs(): SpecIdentity[] {
        return Array.from(this.specs.values());
    }
    
    /**
     * Gets unique spec count (ensures same canonical spec appearing in multiple map placements is counted once)
     */
    public getUniqueSpecCount(): number {
        const uniqueSpecs = new Set<string>();
        
        for (const spec of this.specs.values()) {
            // Create a unique key based on canonical identity
            const uniqueKey = `${spec.repository}:${spec.path}:${spec.version}`;
            uniqueSpecs.add(uniqueKey);
        }
        
        return uniqueSpecs.size;
    }
}

// Export singleton instance
export const specManager = new SpecManager();

// Helper imports (assuming Node.js environment)
import * as fs from 'fs';
import * as path from 'path';
import * as filesystem from 'fs';