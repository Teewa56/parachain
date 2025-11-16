export function Sidebar({ currentPage, onPageChange }: any) {
  const pages = [
    { id: 'dashboard', label: 'Dashboard', icon: '📊' },
    { id: 'issue', label: 'Issue Credential', icon: '✍️' },
    { id: 'credentials', label: 'Credentials', icon: '📜' },
    { id: 'governance', label: 'Governance', icon: '🗳️' },
    { id: 'explorer', label: 'Explorer', icon: '🔍' },
  ];

  return (
    <aside className="w-64 bg-black/30 border-r border-slate-700 p-6 overflow-y-auto">
      <nav className="space-y-2">
        {pages.map((page) => (
          <button
            key={page.id}
            onClick={() => onPageChange(page.id)}
            className={`w-full text-left px-4 py-3 rounded-lg transition ${
              currentPage === page.id
                ? 'bg-blue-600/30 text-blue-300 border border-blue-500/50'
                : 'text-slate-400 hover:text-slate-300 hover:bg-slate-700/20'
            }`}
          >
            <span className="mr-2">{page.icon}</span>
            {page.label}
          </button>
        ))}
      </nav>
    </aside>
  );
}
