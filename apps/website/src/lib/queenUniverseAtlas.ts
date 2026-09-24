/**
 * Queen Universe Atlas
 * 
 * Manages the universe of specs, their states, and provides color coding
 * for visual representation in the Queen system.
 */

import { SpecIdentity, SpecState, SpecAvailability } from './specCatalog';
import { specManager, validateSpecIdentity, SourcePathValidation } from './sharedSpecCore';

export interface SpecColorInfo {
    color: string;
    state: SpecState;
    availability: SpecAvailability;
    message: string;
    isStale: boolean;
    hasError: boolean;
}

export interface AsyncResult<T> {
    data: T | null;
    error: string | null;
    isStale: boolean;
    timestamp: number;
    lastUpdated: number;
}

/**
 * Gets color information for a spec based on its state and availability
 * Addresses stale async result handling with explicit unavailable state
 */
export function getSpecColor(specIdentity: SpecIdentity): SpecColorInfo {
    // Validate spec identity first
    const validation = validateSpecIdentity(specIdentity);
    
    if (!validation.isValid) {
        return {
            color: '#FF0000', // Red for invalid/missing specs
            state: SpecState.Error,
            availability: SpecAvailability.Unavailable,
            message: `Invalid spec: ${validation.errors.join(', ')}`,
            isStale: false,
            hasError: true
        };
    }
    
    // Check source path validation
    if (specIdentity.sourcePath) {
        const sourceValidation: SourcePathValidation = {
            exists: true,
            isAccessible: true,
            isAmbiguous: false,
            resolvedPath: specIdentity.sourcePath
        };
        
        // In a real implementation, this would actually validate the path
        // For now, we'll simulate the validation
        try {
            const fs = require('fs');
            if (specIdentity.sourcePath) {
                sourceValidation.exists = fs.existsSync(specIdentity.sourcePath);
                if (sourceValidation.exists) {
                    sourceValidation.isAccessible = fs.accessSync(specIdentity.sourcePath, fs.constants.R_OK);
                }
            }
        } catch (error) {
            sourceValidation.exists = false;
            sourceValidation.isAccessible = false;
        }
        
        if (!sourceValidation.exists) {
            return {
                color: '#FF0000', // Red for missing source paths
                state: SpecState.Error,
                availability: SpecAvailability.Unavailable,
                message: `Source path does not exist: ${specIdentity.sourcePath}`,
                isStale: false,
                hasError: true
            };
        }
        
        if (sourceValidation.isAmbiguous) {
            return {
                color: '#FFA500', // Orange for ambiguous paths
                state: SpecState.Warning,
                availability: SpecAvailability.Partial,
                message: `Ambiguous source path: ${specIdentity.sourcePath}`,
                isStale: false,
                hasError: true
            };
        }
        
        if (!sourceValidation.isAccessible) {
            return {
                color: '#FF0000', // Red for inaccessible paths
                state: SpecState.Error,
                availability: SpecAvailability.Unavailable,
                message: `Source path not accessible: ${specIdentity.sourcePath}`,
                isStale: false,
                hasError: true
            };
        }
    }
    
    // Get spec state from manager
    const specState = specManager.getSpecState(specIdentity.id);
    
    // Handle stale async results with explicit unavailable state
    const asyncResult = getAsyncSpecResult(specIdentity.id);
    if (asyncResult.isStale) {
        return {
            color: '#FF0000', // Red for stale results (explicit unavailable state)
            state: SpecState.Error,
            availability: SpecAvailability.Unavailable,
            message: `Stale async result for spec: ${specIdentity.id}`,
            isStale: true,
            hasError: true
        };
    }
    
    // Default color coding based on spec state
    switch (specState || SpecState.Unknown) {
        case SpecState.Available:
            return {
                color: '#00FF00', // Green for available specs
                state: SpecState.Available,
                availability: SpecAvailability.Available,
                message: 'Spec is available',
                isStale: false,
                hasError: false
            };
            
        case SpecState.Processing:
            return {
                color: '#FFFF00', // Yellow for processing specs
                state: SpecState.Processing,
                availability: SpecAvailability.Processing,
                message: 'Spec is being processed',
                isStale: false,
                hasError: false
            };
            
        case SpecState.Warning:
            return {
                color: '#FFA500', // Orange for warning specs
                state: SpecState.Warning,
                availability: SpecAvailability.Partial,
                message: 'Spec has warnings',
                isStale: false,
                hasError: false
            };
            
        case SpecState.Error:
            return {
                color: '#FF0000', // Red for error specs
                state: SpecState.Error,
                availability: SpecAvailability.Unavailable,
                message: 'Spec has errors',
                isStale: false,
                hasError: true
            };
            
        case SpecState.Unknown:
            return {
                color: '#808080', // Gray for unknown specs
                state: SpecState.Unknown,
                availability: SpecAvailability.Unknown,
                message: 'Spec state is unknown',
                isStale: false,
                hasError: false
            };
            
        default:
            return {
                color: '#808080', // Gray as fallback
                state: SpecState.Unknown,
                availability: SpecAvailability.Unknown,
                message: 'Unknown spec state',
                isStale: false,
                hasError: false
            };
    }
}

/**
 * Manages async results for specs with staleness tracking
 */
export class AsyncSpecResultManager<T> {
    private results: Map<string, AsyncResult<T>> = new Map();
    private readonly staleTimeout: number;
    
    constructor(staleTimeoutMs: number = 30000) { // Default 30 seconds
        this.staleTimeout = staleTimeoutMs;
    }
    
    /**
     * Sets or updates an async result
     */
    public setResult(specId: string, data: T | null, error: string | null = null): void {
        const result: AsyncResult<T> = {
            data,
            error,
            isStale: false,
            timestamp: Date.now(),
            lastUpdated: Date.now()
        };
        
        this.results.set(specId, result);
    }
    
    /**
     * Gets an async result with staleness checking
     */
    public getResult(specId: string): AsyncResult<T> {
        const existing = this.results.get(specId);
        
        if (!existing) {
            return {
                data: null,
                error: 'No result available',
                isStale: false,
                timestamp: Date.now(),
                lastUpdated: Date.now()
            };
        }
        
        // Check if result is stale
        const isStale = Date.now() - existing.lastUpdated > this.staleTimeout;
        
        return {
            ...existing,
            isStale
        };
    }
    
    /**
     * Updates an existing result without changing staleness status
     */
    public updateResult(specId: string, data: T | null, error: string | null = null): void {
        const existing = this.results.get(specId);
        const lastUpdated = existing ? existing.lastUpdated : Date.now();
        
        const result: AsyncResult<T> = {
            data,
            error,
            isStale: false,
            timestamp: Date.now(),
            lastUpdated
        };
        
        this.results.set(specId, result);
    }
    
    /**
     * Clears stale results
     */
    public clearStaleResults(): void {
        const now = Date.now();
        for (const [specId, result] of this.results.entries()) {
            if (now - result.lastUpdated > this.staleTimeout) {
                this.results.delete(specId);
            }
        }
    }
    
    /**
     * Gets all stale result IDs
     */
    public getStaleResultIds(): string[] {
        const now = Date.now();
        const staleIds: string[] = [];
        
        for (const [specId, result] of this.results.entries()) {
            if (now - result.lastUpdated > this.staleTimeout) {
                staleIds.push(specId);
            }
        }
        
        return staleIds;
    }
}

// Export singleton instance for spec results
export const asyncSpecResultManager = new AsyncSpecResultManager<any>();

/**
 * Helper function to get async spec result
 */
function getAsyncSpecResult(specId: string): AsyncResult<any> {
    return asyncSpecResultManager.getResult(specId);
}

/**
 * Universe Atlas for managing all specs and their states
 */
export class UniverseAtlas {
    private specColors: Map<string, SpecColorInfo> = new Map();
    
    /**
     * Updates spec information and recalculates color
     */
    public updateSpec(specIdentity: SpecIdentity): void {
        const colorInfo = getSpecColor(specIdentity);
        this.specColors.set(specIdentity.id, colorInfo);
        
        // Update spec manager
        specManager.registerSpec(specIdentity);
    }
    
    /**
     * Gets spec color information
     */
    public getSpecColorInfo(specId: string): SpecColorInfo | undefined {
        return this.specColors.get(specId);
    }
    
    /**
     * Gets all spec color information
     */
    public getAllSpecColors(): SpecColorInfo[] {
        return Array.from(this.specColors.values());
    }
    
    /**
     * Gets specs by color/state
     */
    public getSpecsByState(state: SpecState): SpecIdentity[] {
        return specManager.getAllSpecs().filter(spec => 
            specManager.getSpecState(spec.id) === state
        );
    }
    
    /**
     * Gets specs by availability
     */
    public getSpecsByAvailability(availability: SpecAvailability): SpecIdentity[] {
        return specManager.getAllSpecs().filter(spec => {
            const colorInfo = this.getSpecColorInfo(spec.id);
            return colorInfo && colorInfo.availability === availability;
        });
    }
    
    /**
     * Gets total unique spec count
     */
    public getTotalSpecCount(): number {
        return specManager.getUniqueSpecCount();
    }
    
    /**
     * Gets specs that need attention (errors, warnings, stale)
     */
    public getSpecsNeedingAttention(): SpecIdentity[] {
        const allSpecs = specManager.getAllSpecs();
        const attentionSpecs: SpecIdentity[] = [];
        
        for (const spec of allSpecs) {
            const colorInfo = this.getSpecColorInfo(spec.id);
            if (colorInfo && (colorInfo.hasError || colorInfo.state === SpecState.Warning || colorInfo.isStale)) {
                attentionSpecs.push(spec);
            }
        }
        
        return attentionSpecs;
    }
}

// Export singleton instance
export const universeAtlas = new UniverseAtlas();