import { useState, useCallback } from 'react';

export const useSelection = (allItemIds: number[]) => {
    const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());

    const handleSelect = useCallback((id: number, multi: boolean) => {
        setSelectedIds(prev => {
            const newSelected = new Set(multi ? prev : []);
            if (newSelected.has(id)) {
                newSelected.delete(id);
            } else {
                newSelected.add(id);
            }
            return newSelected;
        });
    }, []);

    const handleSelectAll = useCallback((select: boolean) => {
        if (select) {
            setSelectedIds(new Set(allItemIds));
        } else {
            setSelectedIds(new Set());
        }
    }, [allItemIds]);

    const clearSelection = useCallback(() => {
        setSelectedIds(new Set());
    }, []);

    return {
        selectedIds,
        handleSelect,
        handleSelectAll,
        clearSelection,
        setSelectedIds
    };
};
