import { useQuery, keepPreviousData } from '@tanstack/react-query';
import { fileService, storageService } from '../../services/api';

export const useFilesQuery = (path: string) => {
    return useQuery({
        queryKey: ['files', path],
        queryFn: async () => {
            const response = await fileService.listFiles(path);
            return response.data.data.files;
        },
        placeholderData: keepPreviousData,
    });
};

export const useStorageQuery = () => {
    return useQuery({
        queryKey: ['storage'],
        queryFn: async () => {
            const response = await storageService.getStorageInfo();
            return response.data.data;
        },
    });
};
