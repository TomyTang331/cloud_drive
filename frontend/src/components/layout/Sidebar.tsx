import React from 'react';
import type { StorageInfo, CategoryOption } from '../../types';
import { formatBytes } from '../../utils/format';
import './Sidebar.less';

interface SidebarProps {
    storageInfo: StorageInfo | null;
    selectedCategory: CategoryOption;
    onSelectCategory: (category: CategoryOption) => void;
    onNewFolder: () => void;
}

const Sidebar: React.FC<SidebarProps> = ({
    storageInfo,
    selectedCategory,
    onSelectCategory,
    onNewFolder
}) => {

    return (
        <aside className="sidebar">
            <div className="sidebar-header">
                <div className="logo">★</div>
                <h2>Cloud Drive</h2>
            </div>

            <div className="sidebar-actions">
                <button className="btn-new" onClick={onNewFolder}>
                    <span className="plus-icon">+</span> New
                </button>
            </div>

            <nav className="sidebar-nav">
                <button
                    className={selectedCategory === 'all' ? 'nav-item active' : 'nav-item'}
                    onClick={() => onSelectCategory('all')}
                >
                    <span className="nav-icon">📦</span>
                    All Files
                </button>
                <button
                    className={selectedCategory === 'recent' ? 'nav-item active' : 'nav-item'}
                    onClick={() => onSelectCategory('recent')}
                >
                    <span className="nav-icon">🕐</span>
                    Recent
                </button>
                <button
                    className={selectedCategory === 'images' ? 'nav-item active' : 'nav-item'}
                    onClick={() => onSelectCategory('images')}
                >
                    <span className="nav-icon">🖼️</span>
                    Images
                </button>
                <button
                    className={selectedCategory === 'documents' ? 'nav-item active' : 'nav-item'}
                    onClick={() => onSelectCategory('documents')}
                >
                    <span className="nav-icon">📄</span>
                    Documents
                </button>
                <button
                    className={selectedCategory === 'videos' ? 'nav-item active' : 'nav-item'}
                    onClick={() => onSelectCategory('videos')}
                >
                    <span className="nav-icon">🎬</span>
                    Videos
                </button>
            </nav>

            <div className="sidebar-footer">
                <div className="storage-info">
                    <div className="storage-text">
                        <span>Storage</span>
                        <span className="storage-used">
                            {storageInfo
                                ? `${formatBytes(storageInfo.used_bytes)} / ${formatBytes(storageInfo.total_bytes)}`
                                : 'Loading...'}
                        </span>
                    </div>
                    <div className="storage-bar">
                        <div
                            className="storage-progress"
                            style={{ width: `${storageInfo?.usage_percentage ?? 0}%` }}
                        ></div>
                    </div>
                </div>
            </div>
        </aside>
    );
};

export default Sidebar;
