import React, { useState } from 'react';
import type { AuthResponse } from '../../types';

interface TopNavProps {
    user: AuthResponse | null;
    searchQuery: string;
    onSearchChange: (query: string) => void;
    onLogout: () => void;
}

const TopNav: React.FC<TopNavProps> = ({ user, searchQuery, onSearchChange, onLogout }) => {
    const [searchExpanded, setSearchExpanded] = useState(false);

    return (
        <header className="top-nav">
            <div className={`search-bar ${searchExpanded ? 'expanded' : ''}`}>
                <button
                    className="search-toggle"
                    onClick={() => setSearchExpanded(!searchExpanded)}
                    aria-label="Toggle search"
                >
                    <span className="search-icon">🔍</span>
                </button>
                <input
                    type="text"
                    placeholder="Search files..."
                    value={searchQuery}
                    onChange={(e) => onSearchChange(e.target.value)}
                    onFocus={() => setSearchExpanded(true)}
                    onBlur={() => setSearchExpanded(false)}
                />
            </div>

            <div className="top-nav-actions">
                <div className="user-menu">
                    <span className="user-name">{user?.username}</span>
                    <button className="btn-logout" onClick={onLogout}>
                        Logout
                    </button>
                </div>
            </div>
        </header>
    );
};

export default TopNav;
