import React, { useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import './Menu.less';

export interface MenuProps {
    x: number;
    y: number;
    onClose: () => void;
    children: React.ReactNode;
    className?: string;
}

export const Menu: React.FC<MenuProps> = ({ x, y, onClose, children, className = '' }) => {
    const menuRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
                onClose();
            }
        };

        const handleEscape = (event: KeyboardEvent) => {
            if (event.key === 'Escape') {
                onClose();
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        document.addEventListener('keydown', handleEscape);

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            document.removeEventListener('keydown', handleEscape);
        };
    }, [onClose]);

    // Adjust position to keep menu within viewport
    const style: React.CSSProperties = {
        left: x,
        top: y,
    };

    // Simple viewport check (can be enhanced)
    if (menuRef.current) {
        const rect = menuRef.current.getBoundingClientRect();
        if (x + rect.width > window.innerWidth) {
            style.left = x - rect.width;
        }
        if (y + rect.height > window.innerHeight) {
            style.top = y - rect.height;
        }
    }

    return createPortal(
        <div
            className={`menu-container ${className}`}
            style={style}
            ref={menuRef}
            onClick={(e) => e.stopPropagation()}
            onContextMenu={(e) => e.preventDefault()}
        >
            {children}
        </div>,
        document.body
    );
};
