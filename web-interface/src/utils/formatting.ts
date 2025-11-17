export const formatting = {
    truncateAddress(address: string, start: number = 6, end: number = 4): string {
        if (!address || address.length < start + end) return address;
        return `${address.slice(0, start)}...${address.slice(-end)}`;
    },

    formatTimestamp(timestamp: number): string {
        const date = new Date(timestamp);
        return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        });
    },

    formatTimeAgo(timestamp: number): string {
        const seconds = Math.floor((Date.now() - timestamp) / 1000);
        
        if (seconds < 60) return 'just now';
        if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
        if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
        if (seconds < 604800) return `${Math.floor(seconds / 86400)}d ago`;
        
        return new Date(timestamp).toLocaleDateString();
    },

    formatBalance(balance: string | number, decimals: number = 12): string {
        const value = typeof balance === 'string' ? parseFloat(balance) : balance;
        return (value / Math.pow(10, decimals)).toFixed(4);
    },

    formatHash(hash: string, length: number = 8): string {
        if (!hash.startsWith('0x')) return hash;
        return `${hash.slice(0, length + 2)}...${hash.slice(-length)}`;
    },
};