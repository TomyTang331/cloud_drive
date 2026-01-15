import React from 'react';
import './Menu.less';

export interface MenuItemProps {
    label: string;
    icon?: React.ReactNode;
    onClick?: () => void;
    shortcut?: string;
    danger?: boolean;
    disabled?: boolean;
    className?: string;
    children?: React.ReactNode; // For SubMenu
}

export const MenuItem: React.FC<MenuItemProps> = ({
    label,
    icon,
    onClick,
    shortcut,
    danger = false,
    disabled = false,
    className = '',
    children
}) => {
    return (
        <button
            className={`menu-item ${danger ? 'danger' : ''} ${disabled ? 'disabled' : ''} ${className}`}
            onClick={onClick}
            disabled={disabled}
        >
            <span className="menu-item-icon">{icon}</span>
            <span className="menu-item-label">{label}</span>
            {shortcut && <span className="menu-item-shortcut">{shortcut}</span>}
            {children}
        </button>
    );
};
