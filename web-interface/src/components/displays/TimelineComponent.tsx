interface TimelineEvent {
    id: string;
    type: 'issued' | 'revoked' | 'verified' | 'updated' | 'proposal';
    title: string;
    description: string;
    timestamp: number;
    actor?: string;
    metadata?: Record<string, any>;
}

interface TimelineComponentProps {
    events: TimelineEvent[];
    maxEvents?: number;
}

export function TimelineComponent({ events, maxEvents = 10 }: TimelineComponentProps) {
    const sortedEvents = [...events]
        .sort((a, b) => b.timestamp - a.timestamp)
        .slice(0, maxEvents);

    const getEventIcon = (type: TimelineEvent['type']) => {
        switch (type) {
        case 'issued':
            return '✅';
        case 'revoked':
            return '🚫';
        case 'verified':
            return '🔍';
        case 'updated':
            return '🔄';
        case 'proposal':
            return '🗳️';
        default:
            return '•';
        }
    };

    const getEventColor = (type: TimelineEvent['type']) => {
        switch (type) {
        case 'issued':
            return 'border-green-500 bg-green-500/10';
        case 'revoked':
            return 'border-red-500 bg-red-500/10';
        case 'verified':
            return 'border-blue-500 bg-blue-500/10';
        case 'updated':
            return 'border-yellow-500 bg-yellow-500/10';
        case 'proposal':
            return 'border-purple-500 bg-purple-500/10';
        default:
            return 'border-slate-500 bg-slate-500/10';
        }
    };

    const formatTimestamp = (timestamp: number) => {
        const date = new Date(timestamp);
        const now = Date.now();
        const diff = now - timestamp;

        if (diff < 60000) return 'Just now';
        if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
        if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
        if (diff < 604800000) return `${Math.floor(diff / 86400000)}d ago`;
        
        return date.toLocaleDateString('en-US', { 
        month: 'short', 
        day: 'numeric',
        year: date.getFullYear() !== new Date().getFullYear() ? 'numeric' : undefined
        });
    };

    if (sortedEvents.length === 0) {
        return (
        <div className="text-center py-12 text-slate-500">
            <div className="text-4xl mb-3">📭</div>
            <p>No activity yet</p>
        </div>
        );
    }

    return (
        <div className="space-y-4">
        {sortedEvents.map((event, index) => (
            <div key={event.id} className="flex gap-4">
            {/* Timeline Line */}
            <div className="flex flex-col items-center">
                <div className={`w-10 h-10 rounded-full border-2 flex items-center justify-center text-lg ${getEventColor(event.type)}`}>
                {getEventIcon(event.type)}
                </div>
                {index < sortedEvents.length - 1 && (
                <div className="w-0.5 h-full bg-slate-700 mt-2" />
                )}
            </div>

            {/* Event Content */}
            <div className="flex-1 pb-8">
                <div className="bg-slate-800/30 border border-slate-700 rounded-lg p-4 hover:border-slate-600 transition">
                <div className="flex items-start justify-between mb-2">
                    <h4 className="font-semibold text-white">{event.title}</h4>
                    <span className="text-xs text-slate-500 whitespace-nowrap ml-3">
                    {formatTimestamp(event.timestamp)}
                    </span>
                </div>
                <p className="text-sm text-slate-400 mb-2">{event.description}</p>
                {event.actor && (
                    <div className="text-xs text-slate-500">
                    by <span className="font-mono text-slate-400">{event.actor}</span>
                    </div>
                )}
                {event.metadata && Object.keys(event.metadata).length > 0 && (
                    <div className="mt-3 pt-3 border-t border-slate-700">
                    <div className="grid grid-cols-2 gap-2 text-xs">
                        {Object.entries(event.metadata).map(([key, value]) => (
                        <div key={key}>
                            <span className="text-slate-500">{key}:</span>{' '}
                            <span className="text-slate-300">{String(value)}</span>
                        </div>
                        ))}
                    </div>
                    </div>
                )}
                </div>
            </div>
            </div>
        ))}
        </div>
    );
}