# frozen_string_literal: true

module EdgeQuake
  class Client
    SERVICES = {
      health: HealthService,
      documents: DocumentService,
      entities: EntityService,
      relationships: RelationshipService,
      graph: GraphService,
      query: QueryService,
      chat: ChatService,
      tenants: TenantService,
      users: UserService,
      api_keys: ApiKeyService,
      tasks: TaskService,
      pipeline: PipelineService,
      models: ModelService,
      costs: CostService,
      conversations: ConversationService,
      folders: FolderService,
      lineage: LineageService,
      auth: AuthService,
      workspaces: WorkspaceService,
      shared: SharedService,
      connections: ConnectionService,
      operations: OperationsGenerated
    }.freeze

    attr_reader(*SERVICES.keys)

    def initialize(config: Config.new, http: nil)
      transport = http || HttpHelper.new(config)
      SERVICES.each do |name, service|
        instance_variable_set("@#{name}", service.new(transport))
      end
    end
  end
end
