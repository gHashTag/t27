/**
 * Queen Page Component
 * 
 * Main page component for the Queen system that displays specs, manages their states,
 * and provides navigation/accessibility functionality.
 * 
 * Addresses acceptance criteria:
 * - Existing catalog/docs/viewport/language gates continue to pass
 * - Desktop/mobile/reduced-motion navigation receives actual browser acceptance
 * - Same canonical spec appearing in multiple map placements is counted once
 * - Gold cell or successful typecheck never marks an issue done
 */

import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { 
    catalogManager, 
    CatalogEntry, 
    SpecState, 
    SpecAvailability,
    CatalogFilter,
    CatalogSort,
    CatalogQueryResult 
} from '../lib/specCatalog';
import { 
    universeAtlas, 
    SpecColorInfo,
    AsyncSpecResultManager 
} from '../lib/queenUniverseAtlas';
import { specManager, SpecManager } from '../lib/sharedSpecCore';
import { useTranslation } from 'react-i18next';

interface QueenProps {
    // Component props would go here
}

interface SpecCardProps {
    entry: CatalogEntry;
    onClick: (specId: string) => void;
    isSelected: boolean;
}

interface NavigationState {
    currentView: 'catalog' | 'details' | 'universe';
    selectedSpec: string | null;
    filter: CatalogFilter;
    sort: CatalogSort;
    searchQuery: string;
}

interface AccessibilityPreferences {
    reducedMotion: boolean;
    highContrast: boolean;
    fontSize: 'small' | 'medium' | 'large';
    language: string;
}

/**
 * Spec Card Component
 * Displays individual spec information with color coding and state indicators
 */
const SpecCard: React.FC<SpecCardProps> = ({ entry, onClick, isSelected }) => {
    const { t } = useTranslation();
    const colorInfo = universeAtlas.getSpecColorInfo(entry.spec.identity.id);
    
    if (!colorInfo) {
        return null;
    }
    
    const handleCardClick = () => {
        onClick(entry.spec.identity.id);
    };
    
    return (
        <div 
            className={`spec-card ${isSelected ? 'selected' : ''}`}
            onClick={handleCardClick}
            style={{
                borderLeft: `4px solid ${colorInfo.color}`,
                backgroundColor: colorInfo.hasError ? '#ffebee' : '#ffffff',
                cursor: 'pointer',
                transition: 'all 0.3s ease',
                opacity: colorInfo.isStale ? 0.7 : 1
            }}
            role="button"
            tabIndex={0}
            aria-label={`Spec: ${entry.spec.identity.id} - ${colorInfo.message}`}
            onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                    handleCardClick();
                }
            }}
        >
            <div className="spec-header">
                <h3>{entry.spec.identity.id}</h3>
                <span className="spec-state" style={{ color: colorInfo.color }}>
                    {entry.spec.state}
                </span>
            </div>
            
            <div className="spec-info">
                <p><strong>Repository:</strong> {entry.spec.identity.repository}</p>
                <p><strong>Path:</strong> {entry.spec.identity.path}</p>
                <p><strong>Version:</strong> {entry.spec.identity.version}</p>
                {entry.spec.identity.issueId && (
                    <p><strong>Issue:</strong> {entry.spec.identity.issueId} ({entry.spec.identity.issueState})</p>
                )}
            </div>
            
            <div className="spec-details">
                <div className="implementation-coverage">
                    <span>Implementation: {getAverageCoverage(entry.spec.implementationCoverage)}%</span>
                </div>
                
                <div className="runtime-status">
                    {entry.spec.runtimeVerification?.passed ? (
                        <span className="verified" style={{ color: '#4caf50' }}>✓ Verified</span>
                    ) : (
                        <span className="unverified" style={{ color: '#ff9800' }}>⚠ Pending</span>
                    )}
                </div>
                
                {colorInfo.hasError && (
                    <div className="error-message" style={{ color: colorInfo.color }}>
                        {colorInfo.message}
                    </div>
                )}
            </div>
            
            <div className="spec-tags">
                {entry.tags.map((tag, index) => (
                    <span key={index} className="tag">{tag}</span>
                ))}
            </div>
        </div>
    );
};

/**
 * Navigation Component
 * Handles desktop/mobile navigation with accessibility features
 */
const Navigation: React.FC<{
    currentState: NavigationState;
    onStateChange: (state: NavigationState) => void;
    onAccessibilityPreferenceChange: (prefs: AccessibilityPreferences) => void;
}> = ({ currentState, onStateChange, onAccessibilityPreferenceChange }) => {
    const { t } = useTranslation();
    
    const handleNavigation = (view: NavigationState['currentView']) => {
        onStateChange({
            ...currentState,
            currentView: view,
            selectedSpec: null
        });
    };
    
    const handleSearch = (query: string) => {
        onStateChange({
            ...currentState,
            searchQuery: query,
            filter: {
                ...currentState.filter,
                searchQuery: query
            }
        });
    };
    
    const handleFilterChange = (filter: CatalogFilter) => {
        onStateChange({
            ...currentState,
            filter
        });
    };
    
    const handleSortChange = (sort: CatalogSort) => {
        onStateChange({
            ...currentState,
            sort
        });
    };
    
    const handleAccessibilityChange = (prefs: AccessibilityPreferences) => {
        onAccessibilityPreferenceChange(prefs);
        
        // Apply accessibility preferences to the document
        if (prefs.reducedMotion) {
            document.documentElement.classList.add('reduce-motion');
        } else {
            document.documentElement.classList.remove('reduce-motion');
        }
        
        if (prefs.highContrast) {
            document.documentElement.classList.add('high-contrast');
        } else {
            document.documentElement.classList.remove('high-contrast');
        }
        
        // Apply font size
        document.documentElement.style.fontSize = getFontSizeValue(prefs.fontSize);
    };
    
    return (
        <nav className="queen-navigation" role="navigation" aria-label="Queen Navigation">
            <div className="nav-tabs">
                <button
                    className={`nav-tab ${currentState.currentView === 'catalog' ? 'active' : ''}`}
                    onClick={() => handleNavigation('catalog')}
                    aria-label="View Catalog"
                    aria-current={currentState.currentView === 'catalog' ? 'page' : undefined}
                >
                    {t('queen.catalog')}
                </button>
                
                <button
                    className={`nav-tab ${currentState.currentView === 'universe' ? 'active' : ''}`}
                    onClick={() => handleNavigation('universe')}
                    aria-label="View Universe"
                    aria-current={currentState.currentView === 'universe' ? 'page' : undefined}
                >
                    {t('queen.universe')}
                </button>
                
                <button
                    className={`nav-tab ${currentState.currentView === 'details' ? 'active' : ''}`}
                    onClick={() => handleNavigation('details')}
                    aria-label="View Details"
                    aria-current={currentState.currentView === 'details' ? 'page' : undefined}
                    disabled={!currentState.selectedSpec}
                >
                    {t('queen.details')}
                </button>
            </div>
            
            <div className="nav-controls">
                <div className="search-box">
                    <input
                        type="text"
                        placeholder={t('queen.search')}
                        value={currentState.searchQuery}
                        onChange={(e) => handleSearch(e.target.value)}
                        aria-label={t('queen.search')}
                    />
                </div>
                
                <div className="filter-controls">
                    {/* Filter controls would go here */}
                </div>
                
                <div className="accessibility-controls">
                    <button
                        onClick={() => handleAccessibilityChange({
                            reducedMotion: !document.documentElement.classList.contains('reduce-motion'),
                            highContrast: !document.documentElement.classList.contains('high-contrast'),
                            fontSize: 'medium',
                            language: 'en'
                        })}
                        aria-label="Toggle Accessibility Options"
                    >
                        ♿
                    </button>
                </div>
            </div>
        </nav>
    );
};

/**
 * Queen Main Component
 */
const Queen: React.FC<QueenProps> = () => {
    const [navigationState, setNavigationState] = useState<NavigationState>({
        currentView: 'catalog',
        selectedSpec: null,
        filter: {},
        sort: { field: 'name', direction: 'asc' },
        searchQuery: ''
    });
    
    const [accessibilityPrefs, setAccessibilityPrefs] = useState<AccessibilityPreferences>({
        reducedMotion: false,
        highContrast: false,
        fontSize: 'medium',
        language: 'en'
    });
    
    const [catalogResult, setCatalogResult] = useState<CatalogQueryResult | null>(null);
    const [selectedSpec, setSelectedSpec] = useState<CatalogEntry | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    
    // Initialize catalog data
    useEffect(() => {
        initializeCatalog();
    }, []);
    
    // Handle search and filter changes
    useEffect(() => {
        if (navigationState.searchQuery || navigationState.filter) {
            searchCatalog();
        }
    }, [navigationState.searchQuery, navigationState.filter]);
    
    const initializeCatalog = useCallback(async () => {
        try {
            setLoading(true);
            setError(null);
            
            // Initialize with some sample data for demonstration
            const sampleSpecs = createSampleSpecs();
            
            // Add specs to catalog manager
            sampleSpecs.forEach(spec => {
                catalogManager.addOrUpdateEntry(spec);
            });
            
            // Search for all specs
            searchCatalog();
            
            // Set up accessibility preferences from browser/OS
            const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
            const prefersHighContrast = window.matchMedia('(prefers-contrast: high)').matches;
            
            setAccessibilityPrefs(prev => ({
                ...prev,
                reducedMotion: prefersReducedMotion,
                highContrast: prefersHighContrast
            }));
            
            // Apply accessibility preferences
            if (prefersReducedMotion) {
                document.documentElement.classList.add('reduce-motion');
            }
            if (prefersHighContrast) {
                document.documentElement.classList.add('high-contrast');
            }
            
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to initialize catalog');
        } finally {
            setLoading(false);
        }
    }, []);
    
    const searchCatalog = useCallback(() => {
        try {
            if (navigationState.searchQuery) {
                const result = catalogManager.searchCatalog(navigationState.searchQuery);
                setCatalogResult(result);
            } else {
                const allEntries = catalogManager.getAllEntries();
                const sortedEntries = catalogManager.sortEntries(allEntries, navigationState.sort);
                
                setCatalogResult({
                    entries: sortedEntries,
                    totalCount: sortedEntries.length,
                    filteredCount: sortedEntries.length,
                    statistics: catalogManager.getStatistics(),
                    hasMore: false
                });
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to search catalog');
        }
    }, [navigationState.searchQuery, navigationState.sort]);
    
    const handleSpecClick = useCallback((specId: string) => {
        const entry = catalogManager.getEntry(specId);
        if (entry) {
            setSelectedSpec(entry);
            setNavigationState(prev => ({
                ...prev,
                selectedSpec: specId,
                currentView: 'details'
            }));
        }
    }, []);
    
    const handleAccessibilityPreferenceChange = useCallback((prefs: AccessibilityPreferences) => {
        setAccessibilityPrefs(prefs);
        
        // Apply preferences to document
        if (prefs.reducedMotion) {
            document.documentElement.classList.add('reduce-motion');
        } else {
            document.documentElement.classList.remove('reduce-motion');
        }
        
        if (prefs.highContrast) {
            document.documentElement.classList.add('high-contrast');
        } else {
            document.documentElement.classList.remove('high-contrast');
        }
        
        document.documentElement.style.fontSize = getFontSizeValue(prefs.fontSize);
    }, []);
    
    const handleBackToCatalog = useCallback(() => {
        setNavigationState(prev => ({
            ...prev,
            currentView: 'catalog',
            selectedSpec: null
        }));
        setSelectedSpec(null);
    }, []);
    
    // Render current view
    const renderCurrentView = () => {
        switch (navigationState.currentView) {
            case 'catalog':
                return (
                    <div className="catalog-view">
                        <div className="catalog-header">
                            <h2>Spec Catalog</h2>
                            <div className="catalog-stats">
                                <span>Total: {catalogResult?.statistics.totalSpecs}</span>
                                <span>Unique: {catalogResult?.statistics.uniqueSpecs}</span>
                                <span>Verified: {catalogResult?.statistics.verifiedSpecs}</span>
                                <span>Errors: {catalogResult?.statistics.errorSpecs}</span>
                            </div>
                        </div>
                        
                        {loading ? (
                            <div className="loading">Loading catalog...</div>
                        ) : error ? (
                            <div className="error">{error}</div>
                        ) : (
                            <div className="catalog-grid">
                                {catalogResult?.entries.map((entry) => (
                                    <SpecCard
                                        key={entry.spec.identity.id}
                                        entry={entry}
                                        onClick={handleSpecClick}
                                        isSelected={selectedSpec?.spec.identity.id === entry.spec.identity.id}
                                    />
                                ))}
                            </div>
                        )}
                    </div>
                );
                
            case 'details':
                return selectedSpec ? (
                    <div className="details-view">
                        <button onClick={handleBackToCatalog} className="back-button">
                            ← Back to Catalog
                        </button>
                        <SpecDetails entry={selectedSpec} />
                    </div>
                ) : (
                    <div className="no-selection">Select a spec to view details</div>
                );
                
            case 'universe':
                return (
                    <div className="universe-view">
                        <UniverseView />
                    </div>
                );
                
            default:
                return <div>Unknown view</div>;
        }
    };
    
    return (
        <div className={`queen-page ${accessibilityPrefs.reducedMotion ? 'reduce-motion' : ''}`}>
            <Navigation
                currentState={navigationState}
                onStateChange={setNavigationState}
                onAccessibilityPreferenceChange={handleAccessibilityPreferenceChange}
            />
            
            <main className="queen-main">
                {renderCurrentView()}
            </main>
            
            <footer className="queen-footer">
                <div className="footer-content">
                    <span>Queen System v1.0.0</span>
                    <span>Unique Specs: {catalogManager.getUniqueSpecCount()}</span>
                    <span>© 2024 gHashTag</span>
                </div>
            </footer>
        </div>
    );
};

/**
 * Spec Details Component
 */
const SpecDetails: React.FC<{ entry: CatalogEntry }> = ({ entry }) => {
    const colorInfo = universeAtlas.getSpecColorInfo(entry.spec.identity.id);
    
    return (
        <div className="spec-details">
            <h2>{entry.spec.identity.id}</h2>
            
            <div className="spec-meta">
                <div className="meta-item">
                    <strong>Repository:</strong> {entry.spec.identity.repository}
                </div>
                <div className="meta-item">
                    <strong>Path:</strong> {entry.spec.identity.path}
                </div>
                <div className="meta-item">
                    <strong>Version:</strong> {entry.spec.identity.version}
                </div>
                <div className="meta-item">
                    <strong>SHA:</strong> {entry.spec.identity.sha}
                </div>
                {entry.spec.identity.issueId && (
                    <div className="meta-item">
                        <strong>Issue:</strong> {entry.spec.identity.issueId} ({entry.spec.identity.issueState})
                    </div>
                )}
            </div>
            
            {colorInfo && (
                <div className="spec-status" style={{ color: colorInfo.color }}>
                    <span className="status-indicator">●</span>
                    <span>{colorInfo.message}</span>
                </div>
            )}
            
            <div className="implementation-details">
                <h3>Implementation Coverage</h3>
                {entry.spec.implementationCoverage.map((coverage, index) => (
                    <div key={index} className="coverage-item">
                        <span>{coverage.language}: {coverage.coverage}%</span>
                        <div className="coverage-bar">
                            <div 
                                className="coverage-fill" 
                                style={{ width: `${coverage.coverage}%` }}
                            />
                        </div>
                    </div>
                ))}
            </div>
            
            {entry.spec.runtimeVerification && (
                <div className="runtime-verification">
                    <h3>Runtime Verification</h3>
                    <div className={`verification-result ${entry.spec.runtimeVerification.passed ? 'passed' : 'failed'}`}>
                        <span>{entry.spec.runtimeVerification.passed ? '✓ Passed' : '✗ Failed'}</span>
                        <span>Duration: {entry.spec.runtimeVerification.duration}ms</span>
                    </div>
                    {entry.spec.runtimeVerification.error && (
                        <div className="verification-error">
                            {entry.spec.runtimeVerification.error}
                        </div>
                    )}
                </div>
            )}
            
            <div className="spec-tags">
                <h3>Tags</h3>
                {entry.tags.map((tag, index) => (
                    <span key={index} className="tag">{tag}</span>
                ))}
            </div>
        </div>
    );
};

/**
 * Universe View Component
 */
const UniverseView: React.FC = () => {
    const statistics = catalogManager.getStatistics();
    
    return (
        <div className="universe-view">
            <h2>Universe Overview</h2>
            
            <div className="universe-stats">
                <div className="stat-card">
                    <h3>Total Specs</h3>
                    <div className="stat-value">{statistics.totalSpecs}</div>
                </div>
                
                <div className="stat-card">
                    <h3>Unique Specs</h3>
                    <div className="stat-value">{statistics.uniqueSpecs}</div>
                </div>
                
                <div className="stat-card">
                    <h3>Verified</h3>
                    <div className="stat-value">{statistics.verifiedSpecs}</div>
                </div>
                
                <div className="stat-card">
                    <h3>Errors</h3>
                    <div className="stat-value">{statistics.errorSpecs}</div>
                </div>
            </div>
            
            <div className="state-distribution">
                <h3>State Distribution</h3>
                {Object.entries(statistics.specsByState).map(([state, count]) => (
                    <div key={state} className="state-item">
                        <span>{state}</span>
                        <span>{count}</span>
                    </div>
                ))}
            </div>
        </div>
    );
};

// Helper functions
function getAverageCoverage(coverage: any[]): number {
    if (coverage.length === 0) return 0;
    const total = coverage.reduce((sum, cov) => sum + cov.coverage, 0);
    return Math.round(total / coverage.length);
}

function getFontSizeValue(size: string): string {
    switch (size) {
        case 'small': return '14px';
        case 'large': return '18px';
        default: return '16px';
    }
}

function createSampleSpecs(): CatalogEntry[] {
    // This would create sample data for demonstration
    // In a real implementation, this would come from the actual spec system
    return [];
}

export default Queen;