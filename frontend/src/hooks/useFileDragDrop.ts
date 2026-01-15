import { useState, useCallback } from 'react';

interface UseFileDragDropProps {
    onUpload: (files: FileList) => Promise<void>;
}

export const useFileDragDrop = ({ onUpload }: UseFileDragDropProps) => {
    const [isDragging, setIsDragging] = useState(false);

    const handleDragOver = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setIsDragging(true);
    }, []);

    const handleDragLeave = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        if (e.currentTarget === e.target) {
            setIsDragging(false);
        }
    }, []);

    const handleDrop = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setIsDragging(false);

        const { files } = e.dataTransfer;
        if (files && files.length > 0) {
            onUpload(files);
        }
    }, [onUpload]);

    return {
        isDragging,
        handleDragOver,
        handleDragLeave,
        handleDrop
    };
};
