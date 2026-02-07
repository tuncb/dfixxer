unit GenericUnarySpacing;

interface

type
  TStructures = TEnumerable < TStructure >;
  TListOfList = TList < TList < Integer > >;

var
  X: Integer;

function ProcessSteps(
  const AStepNumbers: TArray<TStepNr>;
  const ASteps: TArray<TStepData>
): TArray2 < TStructure >;

function LookupStructure(const AStructureType: TStructureType): TOptional<TStructure>;

implementation

function Add(a, b: Integer): Integer;
begin
  X := - 1;
  X := - Foo;
  X := -Foo(1);
  X := + 2;
  X := + Foo;
  X := a * - 2;
  Result := a - b;
end;

function ProcessSteps(
  const AStepNumbers: TArray<TStepNr>;
  const ASteps: TArray<TStepData>
): TArray2 < TStructure >;
begin
  Result := TArray2<TStructure>;
end;

function LookupStructure(const AStructureType: TStructureType): TOptional<TStructure>;
begin
  Result := Default(TOptional<TStructure>);
end;

end.
