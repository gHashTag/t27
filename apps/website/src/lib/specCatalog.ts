/**
 * Spec Catalog
 * 
 * Defines the core types and interfaces for the spec catalog system,
 * including spec identities, states, availability, and source information.
 */

/**
 * Unique identifier for a spec with full revision tracking
 */
export interface SpecIdentity {
    id: string;
    version: string;
    repository: string;
    path: string;
    sha: string;
    issueId?: string;
    issueState?: 'open' | 'closed' | 'merged';
    sourcePath?: string;
    canonicalPath: string;
    createdAt: string;
    updatedAt: string;
    isTestFixture: boolean;
}

/**
 * State of a spec in the system
 */
export enum SpecState {
    Unknown = 'unknown',
    Available = 'available',
    Processing = 'processing',
    Warning = 'warning',
    Error = 'error',
    Deprecated = 'deprecated'
}

/**
 * Availability status of a spec
 */
export enum SpecAvailability {
    Unknown = 'unknown',
    Available = 'available',
    Processing = 'processing',
    Partial = 'partial',
    Unavailable = 'unavailable'
}

/**
 * Source information for a spec
 */
export interface SpecSource {
    repository: string;
    path: string;
    branch?: string;
    commit?: string;
    url?: string;
    isCanonical: boolean;
    isGenerated: boolean;
    implementationCoverage: number; // 0-100
    runtimeVerification: boolean;
    lastVerified?: string;
}

/**
 * Generation eligibility information
 */
export interface GenerationEligibility {
    canGenerate: boolean;
    reason?: string;
    requiresGeneration: boolean;
    generationType?: 'typescript' | 'swift' | 'rust' | 'verilog';
}

/**
 * Implementation coverage information
 */
export interface ImplementationCoverage {
    language: string;
    coverage: number; // 0-100
    files: string[];
    missingFiles: string[];
    totalLines: number;
    implementedLines: number;
}

/**
 * Runtime verification result
 */
export interface RuntimeVerification {
    passed: boolean;
    timestamp: string;
    duration: number;
    error?: string;
    testResults?: Array<{
        testName: string;
        passed: boolean;
        message?: string;
        duration: number;
    }>;
}

/**
 * Issue lifecycle information
 */
export interface IssueLifecycle {
    issueId: string;
    state: 'open' | 'closed' | 'merged';
    createdAt: string;
    updatedAt: string;
    labels: string[];
    assignees?: string[];
    milestone?: string;
    isBlocking: boolean;
    isDependent: boolean;
    relatedIssues: string[];
}

/**
 * Complete spec information including all metadata
 */
export interface SpecInfo {
    identity: SpecIdentity;
    source: SpecSource;
    availability: SpecAvailability;
    state: SpecState;
    generationEligibility: GenerationEligibility;
    implementationCoverage: ImplementationCoverage[];
    runtimeVerification?: RuntimeVerification;
    issueLifecycle?: IssueLifecycle;
    frozenSnapshot?: SpecIdentity;
    createdAt: string;
    updatedAt: string;
}

/**
 * Catalog entry for display and management
 */
export interface CatalogEntry {
    spec: SpecInfo;
    mapPlacements: string[]; // Where this spec appears in various maps
    displayOrder: number;
    isVisible: boolean;
    tags: string[];
    searchKeywords: string[];
}

/**
 * Catalog statistics
 */
export interface CatalogStatistics {
    totalSpecs: number;
    uniqueSpecs: number; // Canonical specs appearing multiple places counted once
    specsByState: Record<SpecState, number>;
    specsByAvailability: Record<SpecAvailability, number>;
    specsByRepository: Record<string, number>;
    totalImplementationCoverage: number;
    verifiedSpecs: number;
    staleSpecs: number;
    errorSpecs: number;
}

/**
 * Catalog filter options
 */
export interface CatalogFilter {
    states?: SpecState[];
    repositories?: string[];
    availability?: SpecAvailability[];
    hasImplementation?: boolean;
    isVerified?: boolean;
    isStale?: boolean;
    searchQuery?: string;
    tags?: string[];
}

/**
 * Catalog sort options
 */
export interface CatalogSort {
    field: 'name' | 'state' | 'availability' | 'lastUpdated' | 'implementationCoverage' | 'verified';
    direction: 'asc' | 'desc';
}

/**
 * Catalog query result
 */
export interface CatalogQueryResult {
    entries: CatalogEntry[];
    totalCount: number;
    filteredCount: number;
    statistics: CatalogStatistics;
    hasMore: boolean;
    cursor?: string;
}

/**
 * Catalog manager for managing spec catalog operations
 */
export class CatalogManager {
    private entries: Map<string, CatalogEntry> = new Map();
    private specIdToEntryId: Map<string, string> = new Map();
    
    /**
     * Adds or updates a catalog entry
     */
    public addOrUpdateEntry(entry: CatalogEntry): void {
        // Check if spec already exists
        const existingEntryId = this.specIdToEntryId.get(entry.spec.identity.id);
        
        if (existingEntryId) {
            // Update existing entry
            const existing = this.entries.get(existingEntryId);
            if (existing) {
                // Merge map placements
                existing.spec = entry.spec;
                existing.mapPlacements = [...new Set([...existing.mapPlacements, ...entry.mapPlacements])];
                existing.tags = [...new Set([...existing.tags, ...entry.tags])];
                existing.searchKeywords = [...new Set([...existing.searchKeywords, ...entry.searchKeywords])];
            }
        } else {
            // Add new entry
            const entryId = `entry_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
            this.entries.set(entryId, entry);
            this.specIdToEntryId.set(entry.spec.identity.id, entryId);
        }
    }
    
    /**
     * Gets a catalog entry by spec ID
     */
    public getEntry(specId: string): CatalogEntry | undefined {
        const entryId = this.specIdToEntryId.get(specId);
        return entryId ? this.entries.get(entryId) : undefined;
    }
    
    /**
     * Gets all catalog entries
     */
    public getAllEntries(): CatalogEntry[] {
        return Array.from(this.entries.values());
    }
    
    /**
     * Gets unique spec count - ensures same canonical spec appearing in multiple map placements is counted once
     */
    public getUniqueSpecCount(): number {
        const uniqueSpecs = new Set<string>();
        
        for (const entry of this.entries.values()) {
            const uniqueKey = `${entry.spec.identity.repository}:${entry.spec.identity.path}:${entry.spec.identity.version}`;
            uniqueSpecs.add(uniqueKey);
        }
        
        return uniqueSpecs.size;
    }
    
    /**
     * Gets catalog statistics
     */
    public getStatistics(): CatalogStatistics {
        const stats: CatalogStatistics = {
            totalSpecs: this.entries.size,
            uniqueSpecs: this.getUniqueSpecCount(),
            specsByState: {} as Record<SpecState, number>,
            specsByAvailability: {} as Record<SpecAvailability, number>,
            specsByRepository: {},
            totalImplementationCoverage: 0,
            verifiedSpecs: 0,
            staleSpecs: 0,
            errorSpecs: 0
        };
        
        // Initialize counters
        for (const state of Object.values(SpecState)) {
            stats.specsByState[state] = 0;
        }
        
        for (const availability of Object.values(SpecAvailability)) {
            stats.specsByAvailability[availability] = 0;
        }
        
        // Count entries
        for (const entry of this.entries.values()) {
            const spec = entry.spec;
            
            // Count by state
            stats.specsByState[spec.state]++;
            
            // Count by availability
            stats.specsByAvailability[spec.availability]++;
            
            // Count by repository
            stats.specsByRepository[spec.identity.repository] = 
                (stats.specsByRepository[spec.identity.repository] || 0) + 1;
            
            // Calculate total implementation coverage
            const avgCoverage = spec.implementationCoverage.reduce((sum, cov) => sum + cov.coverage, 0) / 
                               spec.implementationCoverage.length;
            stats.totalImplementationCoverage += avgCoverage;
            
            // Count verified specs
            if (spec.runtimeVerification?.passed) {
                stats.verifiedSpecs++;
            }
            
            // Count stale specs
            if (spec.source.isGenerated && !spec.runtimeVerification?.passed) {
                stats.staleSpecs++;
            }
            
            // Count error specs
            if (spec.state === SpecState.Error) {
                stats.errorSpecs++;
            }
        }
        
        // Calculate average implementation coverage
        if (this.entries.size > 0) {
            stats.totalImplementationCoverage = stats.totalImplementationCoverage / this.entries.size;
        }
        
        return stats;
    }
    
    /**
     * Filters catalog entries
     */
    public filterEntries(filter: CatalogFilter): CatalogEntry[] {
        let filtered = this.getAllEntries();
        
        if (filter.states && filter.states.length > 0) {
            filtered = filtered.filter(entry => filter.states!.includes(entry.spec.state));
        }
        
        if (filter.repositories && filter.repositories.length > 0) {
            filtered = filtered.filter(entry => 
                filter.repositories!.includes(entry.spec.identity.repository)
            );
        }
        
        if (filter.availability && filter.availability.length > 0) {
            filtered = filtered.filter(entry => 
                filter.availability!.includes(entry.spec.availability)
            );
        }
        
        if (filter.hasImplementation !== undefined) {
            filtered = filtered.filter(entry => {
                const hasImplementation = entry.spec.implementationCoverage.length > 0;
                return filter.hasImplementation ? hasImplementation : !hasImplementation;
            });
        }
        
        if (filter.isVerified !== undefined) {
            filtered = filtered.filter(entry => {
                const isVerified = entry.spec.runtimeVerification?.passed === true;
                return filter.isVerified ? isVerified : !isVerified;
            });
        }
        
        if (filter.isStale !== undefined) {
            filtered = filtered.filter(entry => {
                const isStale = entry.spec.source.isGenerated && !entry.spec.runtimeVerification?.passed;
                return filter.isStale ? isStale : !isStale;
            });
        }
        
        if (filter.searchQuery) {
            const query = filter.searchQuery.toLowerCase();
            filtered = filtered.filter(entry => {
                const searchText = [
                    entry.spec.identity.id,
                    entry.spec.identity.path,
                    entry.spec.identity.repository,
                    ...entry.tags,
                    ...entry.searchKeywords
                ].join(' ').toLowerCase();
                
                return searchText.includes(query);
            });
        }
        
        if (filter.tags && filter.tags.length > 0) {
            filtered = filtered.filter(entry => 
                filter.tags!.some(tag => entry.tags.includes(tag))
            );
        }
        
        return filtered;
    }
    
    /**
     * Sorts catalog entries
     */
    public sortEntries(entries: CatalogEntry[], sort: CatalogSort): CatalogEntry[] {
        return [...entries].sort((a, b) => {
            let valueA: any, valueB: any;
            
            switch (sort.field) {
                case 'name':
                    valueA = a.spec.identity.id;
                    valueB = b.spec.identity.id;
                    break;
                case 'state':
                    valueA = a.spec.state;
                    valueB = b.spec.state;
                    break;
                case 'availability':
                    valueA = a.spec.availability;
                    valueB = b.spec.availability;
                    break;
                case 'lastUpdated':
                    valueA = new Date(a.spec.updatedAt);
                    valueB = new Date(b.spec.updatedAt);
                    break;
                case 'implementationCoverage':
                    const coverageA = a.spec.implementationCoverage.length > 0 
                        ? a.spec.implementationCoverage.reduce((sum, cov) => sum + cov.coverage, 0) / a.spec.implementationCoverage.length
                        : 0;
                    const coverageB = b.spec.implementationCoverage.length > 0 
                        ? b.spec.implementationCoverage.reduce((sum, cov) => sum + cov.coverage, 0) / b.spec.implementationCoverage.length
                        : 0;
                    valueA = coverageA;
                    valueB = coverageB;
                    break;
                case 'verified':
                    valueA = a.spec.runtimeVerification?.passed ? 1 : 0;
                    valueB = b.spec.runtimeVerification?.passed ? 1 : 0;
                    break;
                default:
                    valueA = a.spec.identity.id;
                    valueB = b.spec.identity.id;
            }
            
            if (valueA < valueB) return sort.direction === 'asc' ? -1 : 1;
            if (valueA > valueB) return sort.direction === 'asc' ? 1 : -1;
            return 0;
        });
    }
    
    /**
     * Searches catalog entries
     */
    public searchCatalog(query: string, limit?: number): CatalogQueryResult {
        const filter: CatalogFilter = {
            searchQuery: query
        };
        
        const entries = this.filterEntries(filter);
        const sortedEntries = this.sortEntries(entries, {
            field: 'relevance',
            direction: 'desc'
        });
        
        const result = {
            entries: limit ? sortedEntries.slice(0, limit) : sortedEntries,
            totalCount: this.entries.size,
            filteredCount: entries.length,
            statistics: this.getStatistics(),
            hasMore: limit && entries.length > limit,
            cursor: limit && entries.length > limit ? `limit_${limit}` : undefined
        };
        
        return result;
    }
    
    /**
     * Gets entries by map placement
     */
    public getEntriesByMapPlacement(placement: string): CatalogEntry[] {
        return this.getAllEntries().filter(entry => 
            entry.mapPlacements.includes(placement)
        );
    }
    
    /**
     * Removes an entry by spec ID
     */
    public removeEntry(specId: string): boolean {
        const entryId = this.specIdToEntryId.get(specId);
        if (entryId) {
            this.entries.delete(entryId);
            this.specIdToEntryId.delete(specId);
            return true;
        }
        return false;
    }
    
    /**
     * Clears all entries
     */
    public clearAll(): void {
        this.entries.clear();
        this.specIdToEntryId.clear();
    }
}

// Export singleton instance
export const catalogManager = new CatalogManager();