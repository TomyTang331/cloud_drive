import React from 'react';
import './Breadcrumbs.less';

interface BreadcrumbsProps {
    currentPath: string;
    onNavigate: (path: string) => void;
}

const Breadcrumbs: React.FC<BreadcrumbsProps> = ({ currentPath, onNavigate }) => {
    const getBreadcrumbs = () => {
        if (currentPath === '/') return [{ name: 'My Files', path: '/' }];

        const parts = currentPath.split('/').filter(p => p);
        const breadcrumbs = [{ name: 'My Files', path: '/' }];

        let accumulatedPath = '';
        for (const part of parts) {
            accumulatedPath += '/' + part;
            breadcrumbs.push({ name: part, path: accumulatedPath });
        }

        return breadcrumbs;
    };

    const breadcrumbs = getBreadcrumbs();

    return (
        <div className="breadcrumb-container">
            {breadcrumbs.map((crumb, index) => (
                <React.Fragment key={crumb.path}>
                    <button
                        className={`breadcrumb-item ${index === breadcrumbs.length - 1 ? 'active' : ''}`}
                        onClick={() => onNavigate(crumb.path)}
                    >
                        {crumb.name}
                    </button>
                    {index < breadcrumbs.length - 1 && (
                        <span className="breadcrumb-separator">/</span>
                    )}
                </React.Fragment>
            ))}
        </div>
    );
};

export default Breadcrumbs;
