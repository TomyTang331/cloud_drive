import React from 'react';
import { useToast } from '../../hooks/useToast';

interface ToolbarProps {
    currentPath: string;
    onNavigate: (path: string) => void;
    viewMode: 'grid' | 'list';
    onViewModeChange: (mode: 'grid' | 'list') => void;
}

const Toolbar: React.FC<ToolbarProps> = ({
    currentPath,
    onNavigate,
    viewMode,
    onViewModeChange
}) => {
    const toast = useToast();

    const handleBreadcrumbClick = (index: number, parts: string[]) => {
        const newPath = parts.slice(0, index + 1).join('/');
        onNavigate(newPath || '/');
    };

    const pathParts = currentPath.split('/').filter(p => p);

    return (
        <div className="toolbar">
            <div className="toolbar-left">
                <button className="btn-primary-action" onClick={() => document.getElementById('file-upload')?.click()}>
                    <span className="icon">↑</span> Upload
                </button>
                <button className="btn-secondary-action" onClick={() => document.dispatchEvent(new CustomEvent('create-folder'))}>
                    <span className="icon">+</span> New Folder
                </button>
                <button className="btn-secondary-action" onClick={() => toast.info('New File feature coming soon')}>
                    <span className="icon">📄</span> New File
                </button>
                <button className="btn-secondary-action" onClick={() => toast.info('Cloud Add feature coming soon')}>
                    <span className="icon">☁️</span> Cloud Add
                </button>
            </div>

            <div className="breadcrumbs" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', fontSize: '1.1rem', color: '#5f6368' }}>
                <span
                    style={{ cursor: 'pointer', fontWeight: 500, color: pathParts.length === 0 ? '#202124' : 'inherit' }}
                    onClick={() => onNavigate('/')}
                >
                    My Files
                </span>
                {pathParts.map((part, index) => (
                    <React.Fragment key={index}>
                        <span>/</span>
                        <span
                            style={{
                                cursor: 'pointer',
                                fontWeight: index === pathParts.length - 1 ? 600 : 500,
                                color: index === pathParts.length - 1 ? '#202124' : 'inherit'
                            }}
                            onClick={() => handleBreadcrumbClick(index, pathParts)}
                        >
                            {part}
                        </span>
                    </React.Fragment>
                ))}
            </div>

            <div className="view-controls">
                <button
                    className={`view-btn ${viewMode === 'grid' ? 'active' : ''}`}
                    onClick={() => onViewModeChange('grid')}
                    title="Grid View"
                >
                    ⊞
                </button>
                <button
                    className={`view-btn ${viewMode === 'list' ? 'active' : ''}`}
                    onClick={() => onViewModeChange('list')}
                    title="List View"
                >
                    ≡
                </button>
                <button className="view-btn" onClick={() => window.location.reload()} title="Refresh">
                    ↻
                </button>
            </div>
        </div>
    );
};

export default Toolbar;
