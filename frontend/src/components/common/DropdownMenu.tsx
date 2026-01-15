import React, { useEffect, useRef } from 'react';
import './DropdownMenu.less';

interface DropdownMenuProps {
    isOpen: boolean;
    onClose: () => void;
    items: {
        label: string;
        icon?: React.ReactNode;
        onClick: () => void;
        danger?: boolean;
        separator?: boolean;
    }[];
    position?: { top: number; left: number } | { top: number; right: number };
}

const DropdownMenu: React.FC<DropdownMenuProps> = ({ isOpen, onClose, items, position }) => {
    const menuRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
                onClose();
            }
        };

        if (isOpen) {
            document.addEventListener('mousedown', handleClickOutside);
        }

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [isOpen, onClose]);

    if (!isOpen) return null;

    return (
        <div
            className="dropdown-menu"
            ref={menuRef}
            style={position ? { position: 'fixed', ...position, zIndex: 1000 } : {}}
        >
            {items.map((item, index) => (
                <React.Fragment key={index}>
                    {item.separator && <div className="dropdown-separator" />}
                    <button
                        className={`dropdown-item ${item.danger ? 'danger' : ''}`}
                        onClick={() => {
                            item.onClick();
                            onClose();
                        }}
                    >
                        {item.icon && <span className="dropdown-icon">{item.icon}</span>}
                        <span className="dropdown-label">{item.label}</span>
                    </button>
                </React.Fragment>
            ))}
        </div>
    );
};

export default DropdownMenu;
