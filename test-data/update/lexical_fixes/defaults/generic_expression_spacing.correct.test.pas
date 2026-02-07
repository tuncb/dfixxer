unit GenericExpressionSpacing;

interface

type
  TMy = class
  public
    procedure Foo();
  end;

function ProcessSteps(
  const AStepNumbers: TArray<TStepNr>;
  const ASteps: TArray<TStepData>
): TArray2<TStructure>;

function LookupStructure(const AStructureType: TStructureType): TOptional<TStructure>;

implementation

procedure TMy.Foo();
var
  LObjects: TList<PObject>;
begin
  LObjects := TList<PObject>.Create();
  LObjects := TDictionary<String, TList<Integer>>.Create();
end;

function ProcessSteps(
  const AStepNumbers: TArray<TStepNr>;
  const ASteps: TArray<TStepData>
): TArray2<TStructure>;
begin
  Result := TArray2<TStructure>;
end;

function LookupStructure(const AStructureType: TStructureType): TOptional<TStructure>;
begin
  Result := Default(TOptional<TStructure>);
end;

end.
