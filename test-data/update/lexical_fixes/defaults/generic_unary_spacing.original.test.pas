unit GenericUnarySpacing;

interface

type
  TStructures = TEnumerable < TStructure >;
  TListOfList = TList < TList < Integer > >;

var
  X: Integer;

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

end.
